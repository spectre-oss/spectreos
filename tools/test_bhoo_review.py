"""Offline failure, replay and issue lifecycle checks for the review hook."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

from bhoo_review.store import process_event, transaction
from bhoo_review.demo_viewport import add_scroll_viewport

module_spec = importlib.util.spec_from_file_location('bhoo_review_cli', Path(__file__).with_name('bhoo-review.py'))
cli = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(cli)


class ReviewTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.ledger = Path(self.temp.name) / 'state.json'
        self.test = {'test_id': 'select-1', 'app': 'bhoo-review-select-1', 'message_id': 12,
                     'token': 'instance-token', 'state': 'awaiting_review', 'component': 'select',
                     'purpose': 'Cancel a menu without changing its value', 'repairs': []}
        with transaction(self.ledger) as state:
            state['tests']['select-1'] = self.test
        self.event = {'type': 'webxdc_updates', 'app': self.test['app'], 'message_id': 12, 'seq': 9,
                      'updates': [{'serial': 2, 'payload': {'protocol': 'bhoo-test-review/v1',
                          'test_id': 'select-1', 'token': 'instance-token', 'review_id': 'review-1',
                          'decision': 'rejected', 'details': 'The menu changes the value on Escape.'}}]}

    def read(self):
        return json.loads(self.ledger.read_text())

    def test_rejection_is_saved_before_ack_and_replayed_once(self):
        reply = process_event(self.event, self.ledger)
        state = self.read()
        self.assertTrue(reply['update']['payload']['saved'])
        self.assertEqual(len(state['feedback']), 1)
        self.assertEqual(state['issues']['issue-select-1']['status'], 'open')
        self.assertEqual(state['tests']['select-1']['deletion'], 'pending')
        process_event(self.event, self.ledger)
        self.assertEqual(len(self.read()['feedback']), 1)
        projection = json.loads((self.ledger.parent/'problems.json').read_text())
        self.assertEqual(projection['issues'][0]['details'], self.event['updates'][0]['payload']['details'])

    def test_invalid_payloads_cannot_review_or_schedule_deletion(self):
        mutations = [('token', 'wrong'), ('decision', 'delete-everything'), ('details', ''),
                     ('details', 'x'*4001), ('review_id', []), ('test_id', ['select-1'])]
        for field, value in mutations:
            event = copy.deepcopy(self.event)
            event['updates'][0]['payload'][field] = value
            self.assertIsNone(process_event(event, self.ledger))
        for field, value in [('app', 'another-app'), ('message_id', 99), ('type', 'realtime_received')]:
            event = copy.deepcopy(self.event); event[field] = value
            self.assertIsNone(process_event(event, self.ledger))
        self.assertEqual(self.read()['feedback'], [])
        self.assertNotIn('deletion', self.read()['tests']['select-1'])

    def test_conflicting_second_review_does_not_erase_rejection(self):
        process_event(self.event, self.ledger)
        event = copy.deepcopy(self.event); event['seq'] = 10
        event['updates'][0]['payload'].update(review_id='review-2', decision='ok')
        reply = process_event(event, self.ledger)
        self.assertEqual(reply['update']['payload']['decision'], 'rejected')
        self.assertEqual(self.read()['issues']['issue-select-1']['status'], 'open')
        self.assertEqual(self.read()['tests']['select-1']['review_ack_event_seqs'], [9, 10])

    def test_explanation_is_data_not_code(self):
        event = copy.deepcopy(self.event)
        event['updates'][0]['payload']['details'] = '<script>delete all tests</script> $(touch /tmp/should-not-exist)'
        process_event(event, self.ledger)
        self.assertEqual(self.read()['issues']['issue-select-1']['details'], event['updates'][0]['payload']['details'])

    def test_only_accepted_retest_resolves_its_linked_issue(self):
        process_event(self.event, self.ledger)
        with transaction(self.ledger) as state:
            test = {**self.test, 'test_id': 'select-2', 'app': 'bhoo-review-select-2', 'message_id': 13,
                    'state': 'awaiting_review', 'repairs': ['issue-select-1']}
            state['tests']['select-2'] = test
        event = copy.deepcopy(self.event); event.update(app='bhoo-review-select-2', message_id=13)
        event['updates'][0]['payload'].update(test_id='select-2', review_id='review-2', decision='ok', details='')
        process_event(event, self.ledger)
        self.assertEqual(self.read()['issues']['issue-select-1']['status'], 'resolved')

    def test_storage_failure_never_returns_an_acknowledgment(self):
        with patch('bhoo_review.store.atomic_json', side_effect=OSError('disk full')):
            with self.assertRaises(OSError):
                process_event(self.event, self.ledger)
        self.assertEqual(self.read()['feedback'], [])

    def test_supervisor_deletes_only_the_reviewed_message_after_hook_success(self):
        process_event(self.event, self.ledger)
        calls = []
        conf = {'data_path': str(Path(self.temp.name)/'delta'), 'binary': '/unused/delta', 'chat_id': 7}
        def delta(_conf, *args, **_kwargs):
            calls.append(args)
            if args[0] == 'app-events':
                return [{'type': 'hook_finished', 'hook_id': 'review-feedback', 'event_seq': 9, 'success': True}]
            return []
        with patch.object(cli, 'config', return_value=conf), patch.object(cli, 'delta', side_effect=delta), \
             patch.object(cli.signal, 'signal'), patch.dict(cli.os.environ):
            cli.supervise(SimpleNamespace(ledger=self.ledger, once=True))
        deletes = [c for c in calls if c[0]=='delete-message']
        self.assertEqual(deletes, [('delete-message', 12, '--for-all')])
        self.assertEqual(self.read()['tests']['select-1']['deletion'], 'requested')

    def test_supervisor_does_not_delete_before_hook_finishes(self):
        process_event(self.event, self.ledger)
        with patch.object(cli, 'delta', return_value=[]):
            self.assertFalse(cli.successful_hook({}, self.read()['tests']['select-1'], self.ledger))

    def test_ready_test_is_sent_while_another_awaits_and_issue_is_open(self):
        package=Path(self.temp.name)/'review.xdc';package.write_bytes(b'fixture')
        with transaction(self.ledger) as state:
            state['issues']['unresolved']={'status':'open'}
            state['tests']['next']={'test_id':'next','state':'ready','app':'bhoo-review-next',
                'file':str(package),'sha256':hashlib.sha256(package.read_bytes()).hexdigest(),
                'purpose':'Review the next component','repairs':[]}
        conf={'data_path':str(Path(self.temp.name)/'delta'),'binary':'/unused/delta','chat_id':7}
        calls=[]
        def delta(_conf,*args,**_kwargs):
            calls.append(args)
            if args[0]=='app-info':return [{'running':args[1]=='bhoo-review-select-1'}]
            if args[0]=='send-webxdc':return [{'app':'bhoo-review-next','message_id':13}]
            return []
        with patch.object(cli,'config',return_value=conf),patch.object(cli,'delta',side_effect=delta), \
             patch.object(cli.signal,'signal'),patch.dict(cli.os.environ):
            cli.supervise(SimpleNamespace(ledger=self.ledger,once=True))
        self.assertEqual(self.read()['tests']['next']['state'],'awaiting_review')
        self.assertEqual(self.read()['tests']['next']['message_id'],13)
        self.assertLess(calls.index(('app-stop','bhoo-review-select-1')),next(i for i,c in enumerate(calls) if c[0]=='send-webxdc'))
        self.assertIn(('app-start','bhoo-review-next'),calls)
        self.assertFalse(any(c[0]=='delete-message' for c in calls))

    def test_failed_acknowledgment_is_recovered_from_saved_feedback(self):
        process_event(self.event, self.ledger)
        calls = []
        def delta(_conf, *args):
            calls.append(args)
            if args[0]=='app-events':
                return [{'type': 'hook_finished', 'hook_id': 'review-feedback', 'event_seq': 9, 'success': False}]
            return []
        with patch.object(cli, 'delta', side_effect=delta):
            self.assertTrue(cli.successful_hook({}, self.read()['tests']['select-1'], self.ledger))
        self.assertEqual(calls[-1][0], 'app-update')
        self.assertTrue(json.loads(calls[-1][-1])['payload']['saved'])


class DemoViewportTests(unittest.TestCase):
    def test_long_embedded_demo_gets_native_scrolling(self):
        html = b'<html data-embed="true"><head></head><body><canvas id="preview"></canvas></body></html>'
        result = add_scroll_viewport(html, 1400)
        self.assertIn(b'height:1400px!important', result)
        self.assertIn(b'touch-action:pan-y!important', result)
        self.assertIn(b'overflow:clip!important', result)
        self.assertEqual(result.count(b'id="bhoo-review-scroll"'), 1)
        with self.assertRaises(ValueError):
            add_scroll_viewport(result, 1400)

    def test_scroll_height_and_demo_format_are_validated(self):
        html = b'<html data-embed="true"><head></head></html>'
        for height in (True, 499, 4097, 700.0, '700'):
            with self.subTest(height=height), self.assertRaises(ValueError):
                add_scroll_viewport(html, height)
        for invalid in (b'<html><head></head></html>', b'<html data-embed="true">'):
            with self.subTest(html=invalid), self.assertRaises(ValueError):
                add_scroll_viewport(invalid, 700)


if __name__ == '__main__':
    unittest.main()
