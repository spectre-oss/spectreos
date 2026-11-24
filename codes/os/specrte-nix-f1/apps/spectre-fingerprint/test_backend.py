import importlib.util
import unittest
import tempfile
import sys
import os
from pathlib import Path

spec = importlib.util.spec_from_file_location('backend', Path(__file__).with_name('backend.py'))
backend = importlib.util.module_from_spec(spec)
spec.loader.exec_module(backend)

class HardwareResults(unittest.TestCase):
    def test_real_geometry_requires_successful_transport_and_otp(self):
        output = 'Sensor geometry=88x108; OTP bytes=32; OTP data=present'
        self.assertTrue(backend.result_for(0, output, .2)['ok'])
        self.assertFalse(backend.result_for(1, output, .2)['ok'])
        self.assertFalse(backend.result_for(0, output.replace('present', 'zero'), .2)['ok'])
        self.assertFalse(backend.result_for(0, output.replace('bytes=32', 'bytes=0'), .2)['ok'])
        self.assertFalse(backend.result_for(0, output.replace('88x108', '0x0'), .2)['ok'])

    def test_missing_test_kernel(self):
        result = backend.result_for(1, 'TEE open: No such file or directory', 0)
        self.assertEqual(result['title'], 'Test kernel needed')
        self.assertFalse(result['ok'])

    def test_power_off_control_is_not_success(self):
        result = backend.result_for(1, 'Vendor status=1062', .2)
        self.assertEqual(result['title'], 'Sensor unavailable')
        self.assertFalse(result['ok'])

    def test_unexpected_output_cannot_report_success(self):
        self.assertFalse(backend.result_for(0, 'Transport ret=0', .1)['ok'])
        self.assertLessEqual(len(backend.result_for(1, 'x' * 20000, .1)['details']), 16000)

class EnrollmentResults(unittest.TestCase):
    def test_list_requires_success_and_a_real_count(self):
        output = 'Group 1000 template count=0'
        self.assertEqual(backend.operation_result('list', 0, output, .2)['enrolled_count'], 0)
        self.assertFalse(backend.operation_result('list', 1, output, .2)['ok'])
        self.assertIsNone(backend.operation_result('list', 0, '', .2)['enrolled_count'])

    def test_authorization_rejection_is_not_enrollment(self):
        result = backend.operation_result('enroll', 1, 'Vendor status=1057', .2)
        self.assertFalse(result['ok'])
        self.assertEqual(result['title'], 'Enrollment authorization needed')

    def test_accepted_start_is_not_a_saved_fingerprint(self):
        self.assertFalse(backend.operation_result('enroll', 0, 'Enrollment start accepted', .2)['ok'])

    def test_count_does_not_claim_a_match(self):
        for count in (0, 1):
            result = backend.operation_result('verify', 0, f'Group 1000 template count={count}', .2)
            self.assertFalse(result['ok'])

class CaptureResults(unittest.TestCase):
    def test_saved_requires_verified_enumeration_and_successful_cleanup(self):
        output = 'SAVE_VERIFIED count=1 saved_template_listed=yes'
        self.assertTrue(backend.operation_result('enroll', 0, output, 1)['ok'])
        self.assertFalse(backend.operation_result('enroll', 1, output, 1)['ok'])
        self.assertFalse(backend.operation_result('enroll', 0, output.replace('yes', 'no'), 1)['ok'])

    def test_accept_and_reject_are_distinct_completed_checks(self):
        for text, matched in [('yes identity=expected', True), ('no', False)]:
            output = 'MATCH_RESULT accepted=' + text
            result = backend.operation_result('verify', 0, output, 1)
            self.assertTrue(result['ok'])
            self.assertEqual(result['matched'], matched)
            self.assertFalse(backend.operation_result('verify', 1, output, 1)['ok'])

    def test_retry_and_inconclusive_are_not_rejections(self):
        for output in ['Vendor status=1064', 'MATCH_RETRY: continuing vendor phase 2',
                       'MATCH_RESULT inconclusive=fast_or_accidental_touch',
                       'MATCH_RESULT accepted=yes identity=wrong']:
            self.assertFalse(backend.operation_result('verify', 0, output, 1)['ok'])

    def test_progress_is_not_completion(self):
        self.assertIsNotNone(backend.progress_for('CAPTURE_READY: touch sensor'))
        self.assertIn('3 samples', backend.progress_for('ENROLL_PROGRESS remaining=3 accepted=yes')['message'])
        self.assertIsNone(backend.progress_for('SAVE_VERIFIED count=1 saved_template_listed=yes'))

class HelperIntegration(unittest.TestCase):
    def test_progress_and_final_result_from_child_process(self):
        with tempfile.TemporaryDirectory() as directory:
            helper = Path(directory) / 'helper'
            helper.write_text('#!' + sys.executable + '\n' +
                "import sys\nassert sys.argv[1:] == ['--save-probe']\n" +
                "print('CAPTURE_READY: touch sensor', flush=True)\n" +
                "print('ENROLL_PROGRESS remaining=1 accepted=yes', flush=True)\n" +
                "print('SAVE_VERIFIED count=1 saved_template_listed=yes', flush=True)\n")
            helper.chmod(0o700)
            events = []
            result = backend.run_test(str(helper), 'enroll', events.append)
            self.assertEqual(len(events), 2)
            self.assertTrue(result['ok'])
            self.assertEqual(result['enrolled_count'], 1)

    def test_real_hardware_transcripts(self):
        evidence = Path(__file__).resolve().parents[2] / '.build/fingerprint/android-comparison/controlled-r5-logs'
        if not evidence.exists():
            self.skipTest('Private local hardware evidence not present')
        for action, name, field, expected in [
            ('enroll', 'enroll', 'enrolled_count', 1),
            ('verify', 'positive', 'matched', True),
            ('verify', 'negative-fixed', 'matched', False),
        ]:
            result = backend.operation_result(action, 0, (evidence / (name + '.txt')).read_text(), 1)
            self.assertTrue(result['ok'])
            self.assertEqual(result[field], expected)

if __name__ == '__main__':
    unittest.main()
