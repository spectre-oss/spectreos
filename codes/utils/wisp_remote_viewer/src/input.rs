use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use winit::keyboard::KeyCode;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pointer {
    Mouse(u8),
    Touch(u64),
}
#[derive(Default)]
pub struct Input {
    pub pointers: HashMap<Pointer, u32>,
    pub keys: HashSet<u32>,
}
impl Input {
    pub fn down(&mut self, pointer: Pointer, point: [f64; 2]) -> Option<Value> {
        if self.pointers.contains_key(&pointer) {
            return None;
        }
        let slot = (0..10).find(|slot| !self.pointers.values().any(|s| s == slot))?;
        self.pointers.insert(pointer, slot);
        Some(json!({"type":"touch_down","slot":slot,"x":point[0],"y":point[1]}))
    }
    pub fn motion(&self, pointer: Pointer, point: [f64; 2]) -> Option<Value> {
        self.pointers
            .get(&pointer)
            .map(|slot| json!({"type":"touch_move","slot":slot,"x":point[0],"y":point[1]}))
    }
    pub fn up(&mut self, pointer: Pointer) -> Option<Value> {
        self.pointers
            .remove(&pointer)
            .map(|slot| json!({"type":"touch_up","slot":slot}))
    }
    pub fn key(&mut self, key: KeyCode, pressed: bool) -> Option<Value> {
        let code = evdev(key)?;
        if pressed {
            if !self.keys.insert(code) {
                return None;
            }
        } else if !self.keys.remove(&code) {
            return None;
        }
        Some(json!({"type":"key","code":code,"down":pressed}))
    }
    pub fn release(&mut self) -> Vec<Value> {
        let mut values = Vec::new();
        if !self.pointers.is_empty() {
            values.push(json!({"type":"touch_cancel"}));
            self.pointers.clear();
        }
        values.extend(
            self.keys
                .drain()
                .map(|code| json!({"type":"key","code":code,"down":false})),
        );
        values
    }
}
pub fn evdev(key: KeyCode) -> Option<u32> {
    use KeyCode::*;
    Some(match key {
        Escape => 1,
        Digit1 => 2,
        Digit2 => 3,
        Digit3 => 4,
        Digit4 => 5,
        Digit5 => 6,
        Digit6 => 7,
        Digit7 => 8,
        Digit8 => 9,
        Digit9 => 10,
        Digit0 => 11,
        Minus => 12,
        Equal => 13,
        Backspace => 14,
        Tab => 15,
        KeyQ => 16,
        KeyW => 17,
        KeyE => 18,
        KeyR => 19,
        KeyT => 20,
        KeyY => 21,
        KeyU => 22,
        KeyI => 23,
        KeyO => 24,
        KeyP => 25,
        BracketLeft => 26,
        BracketRight => 27,
        Enter => 28,
        ControlLeft => 29,
        KeyA => 30,
        KeyS => 31,
        KeyD => 32,
        KeyF => 33,
        KeyG => 34,
        KeyH => 35,
        KeyJ => 36,
        KeyK => 37,
        KeyL => 38,
        Semicolon => 39,
        Quote => 40,
        Backquote => 41,
        ShiftLeft => 42,
        Backslash => 43,
        KeyZ => 44,
        KeyX => 45,
        KeyC => 46,
        KeyV => 47,
        KeyB => 48,
        KeyN => 49,
        KeyM => 50,
        Comma => 51,
        Period => 52,
        Slash => 53,
        ShiftRight => 54,
        NumpadMultiply => 55,
        AltLeft => 56,
        Space => 57,
        CapsLock => 58,
        F1 => 59,
        F2 => 60,
        F3 => 61,
        F4 => 62,
        F5 => 63,
        F6 => 64,
        F7 => 65,
        F8 => 66,
        F9 => 67,
        F10 => 68,
        NumLock => 69,
        ScrollLock => 70,
        Numpad7 => 71,
        Numpad8 => 72,
        Numpad9 => 73,
        NumpadSubtract => 74,
        Numpad4 => 75,
        Numpad5 => 76,
        Numpad6 => 77,
        NumpadAdd => 78,
        Numpad1 => 79,
        Numpad2 => 80,
        Numpad3 => 81,
        Numpad0 => 82,
        NumpadDecimal => 83,
        IntlBackslash => 86,
        F11 => 87,
        F12 => 88,
        NumpadEnter => 96,
        ControlRight => 97,
        NumpadDivide => 98,
        PrintScreen => 99,
        AltRight => 100,
        Home => 102,
        ArrowUp => 103,
        PageUp => 104,
        ArrowLeft => 105,
        ArrowRight => 106,
        End => 107,
        ArrowDown => 108,
        PageDown => 109,
        Insert => 110,
        Delete => 111,
        AudioVolumeMute => 113,
        AudioVolumeDown => 114,
        AudioVolumeUp => 115,
        Pause => 119,
        SuperLeft => 125,
        SuperRight => 126,
        ContextMenu => 127,
        _ => return None,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contacts_and_focus_loss_release_everything_once() {
        let mut input = Input::default();
        assert_eq!(
            input.down(Pointer::Mouse(0), [10.0, 20.0]).unwrap()["slot"],
            0
        );
        assert_eq!(
            input.down(Pointer::Touch(777), [20.0, 20.0]).unwrap()["slot"],
            1
        );
        assert!(input.key(KeyCode::ShiftLeft, true).is_some());
        assert!(input.key(KeyCode::ShiftLeft, true).is_none());
        let release = input.release();
        assert_eq!(release.len(), 2);
        assert_eq!(release[0]["type"], "touch_cancel");
        assert_eq!(release[1]["down"], false);
        assert!(input.release().is_empty());
    }
}
