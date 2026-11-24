"""Validate requests and ensure gestures release every contact, even on failure."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('phone_debug', Path(__file__).resolve().parents[1] / 'debug/phone.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class FakeInput:
    def __init__(self):
        self.events = []
    def event(self, *event):
        self.events.append(event)
    def sync(self):
        self.events.append((0, 0, 0))


class InputTests(unittest.TestCase):
    def test_reject_invalid_frames_before_device_access(self):
        cases = [[], [{'points':[{'id':0,'x':float('nan'),'y':0}]}],
                 [{'points':[{'id':0,'x':0,'y':0}]*2}],
                 [{'points':[], 'delay_ms':1001}]]
        for frames in cases:
            with self.subTest(frames=frames), self.assertRaises(ValueError):
                m.validate_frames(frames)

    def test_two_contacts_are_released(self):
        agent = m.Agent(); agent.touch = FakeInput()
        frames = [{'points':[{'id':0,'x':0.2,'y':0.3},{'id':1,'x':0.8,'y':0.3}], 'delay_ms':0}]
        agent.perform({'action':'touch','frames':frames})
        events = agent.touch.events
        self.assertIn((3,57,1), events); self.assertIn((3,57,2), events)
        self.assertEqual(events.count((3,57,-1)), 2)
        self.assertEqual(events[-3:], [(1,330,0),(1,325,0),(0,0,0)])

    def test_interruption_releases_contacts(self):
        agent = m.Agent(); agent.touch = FakeInput()
        with patch.object(m.time,'sleep',side_effect=KeyboardInterrupt), self.assertRaises(KeyboardInterrupt):
            agent.perform({'action':'touch','frames':[{'points':[{'id':2,'x':.5,'y':.5}]}]})
        self.assertIn((3,57,-1), agent.touch.events)

    def test_locked_session_cannot_force_keyboard(self):
        agent = m.Agent()
        with patch.object(agent, 'screen_locked', return_value=True), patch.object(agent, 'dbus') as dbus:
            with self.assertRaises(ValueError):
                agent.perform({'action':'keyboard','visible':True})
            dbus.assert_not_called()

    def test_debug_keyboard_state_is_restored(self):
        agent = m.Agent()
        with patch.object(agent, 'screen_locked', return_value=False), patch.object(agent, 'dbus', return_value='(<false>,)') as dbus, patch.object(m.time, 'sleep'):
            agent.perform({'action':'keyboard','visible':True})
            agent.close()
            self.assertEqual(dbus.call_args.args, ('sm.puri.OSK0.SetVisible','false'))

    def test_bad_button_has_no_input_side_effects(self):
        agent = m.Agent()
        with self.assertRaises(ValueError):
            agent.perform({'action':'mouse','click':'invalid'})
        self.assertIsNone(agent.mouse)


if __name__ == '__main__':
    unittest.main()
