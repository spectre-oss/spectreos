"""Atomic review ledger. Remote feedback is data, never executable instructions."""
from contextlib import contextmanager
from datetime import datetime, timezone
import fcntl
import json
import os
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_LEDGER = ROOT / 'codes/libs/ui/libbhoo/docs/reviews/state.json'


def now():
    return datetime.now(timezone.utc).isoformat()


def atomic_json(path, data):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(prefix='.' + path.name, dir=path.parent)
    try:
        with os.fdopen(fd, 'w') as out:
            json.dump(data, out, ensure_ascii=False, indent=2)
            out.write('\n')
            out.flush()
            os.fsync(out.fileno())
        os.replace(name, path)
        directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        if os.path.exists(name):
            os.unlink(name)


@contextmanager
def transaction(path=DEFAULT_LEDGER):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with (path.parent / '.review.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        state = json.loads(path.read_text()) if path.exists() else {'schema': 1, 'tests': {}, 'feedback': [], 'issues': {}}
        yield state
        atomic_json(path, state)
        # This projection is repairable from the authoritative ledger after a crash.
        atomic_json(path.parent / 'problems.json', {'schema': 1, 'issues': list(state['issues'].values())})


def process_event(event, ledger=DEFAULT_LEDGER):
    if event.get('type') != 'webxdc_updates':
        return None
    acknowledgment = None
    with transaction(ledger) as state:
        for update in event.get('updates', []):
            payload = update.get('payload')
            if not isinstance(payload, dict) or payload.get('protocol') != 'bhoo-test-review/v1':
                continue
            test_id = payload.get('test_id')
            if not isinstance(test_id, str):
                continue
            test = state['tests'].get(test_id)
            if not test or event.get('app') != test['app'] or event.get('message_id') != test.get('message_id'):
                continue
            if payload.get('token') != test['token']:
                continue
            review_id = payload.get('review_id')
            decision, details = payload.get('decision'), payload.get('details', '')
            if not isinstance(review_id, str) or not 1 <= len(review_id) <= 128:
                continue
            if decision not in ('ok', 'rejected') or not isinstance(details, str) or len(details) > 4000:
                continue
            details = details.strip()
            if decision == 'rejected' and not details:
                continue
            existing = next((r for r in state['feedback'] if r['test_id'] == test_id), None)
            if existing:
                test.setdefault('review_ack_event_seqs', [test.get('review_event_seq')])
                if event.get('seq') not in test['review_ack_event_seqs']:
                    test['review_ack_event_seqs'].append(event.get('seq'))
                # First valid review wins, even if another device submits a conflict.
                acknowledgment = {'protocol': 'bhoo-test-review-ack/v1', 'test_id': test_id,
                                  'review_id': review_id, 'decision': existing['decision'], 'saved': True}
                continue
            if test.get('state') != 'awaiting_review':
                continue
            review = {'test_id': test_id, 'review_id': review_id, 'decision': decision, 'details': details,
                      'received_at': now(), 'event_seq': event.get('seq'), 'update_serial': update.get('serial')}
            state['feedback'].append(review)
            test.update(state='reviewed', decision=decision, review_event_seq=event.get('seq'),
                        reviewed_at=review['received_at'], deletion='pending', review_ack_event_seqs=[event.get('seq')])
            if decision == 'rejected':
                for old_id in test.get('repairs', []):
                    if old_id in state['issues']:
                        state['issues'][old_id]['status'] = 'open'
                issue_id = 'issue-' + test_id
                state['issues'][issue_id] = {'id': issue_id, 'test_id': test_id, 'component': test['component'],
                    'purpose': test['purpose'], 'details': details, 'status': 'open', 'created_at': now(),
                    'retest_ids': []}
            else:
                for issue_id in test.get('repairs', []):
                    issue = state['issues'].get(issue_id)
                    if issue:
                        issue.update(status='resolved', accepted_test=test_id, resolved_at=now())
            acknowledgment = {'protocol': 'bhoo-test-review-ack/v1', 'test_id': test_id,
                              'review_id': review_id, 'decision': decision, 'saved': True}
    return {'update': {'payload': acknowledgment}} if acknowledgment else None
