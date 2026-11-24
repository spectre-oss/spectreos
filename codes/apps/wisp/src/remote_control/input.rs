//! Validate remote input transitions before dispatch into compositor handlers.
use super::protocol::Command;
use std::collections::HashSet;

#[derive(Default)]
pub struct InputState {
    pub contacts: HashSet<u32>,
    pub keys: HashSet<u32>,
}
impl InputState {
    pub fn validate(
        &mut self,
        command: &Command,
        width: f64,
        height: f64,
    ) -> Result<(), &'static str> {
        match *command {
            Command::TouchDown { slot, x, y } | Command::TouchMove { slot, x, y } => {
                if slot > 9
                    || !x.is_finite()
                    || !y.is_finite()
                    || x < 0.0
                    || y < 0.0
                    || x >= width
                    || y >= height
                {
                    return Err("invalid touch slot or coordinates");
                }
                if matches!(command, Command::TouchDown { .. }) {
                    if !self.contacts.insert(slot) {
                        return Err("contact already down");
                    }
                } else if !self.contacts.contains(&slot) {
                    return Err("contact is not down");
                }
            }
            Command::TouchUp { slot } => {
                if !self.contacts.remove(&slot) {
                    return Err("contact is not down");
                }
            }
            Command::TouchCancel => self.contacts.clear(),
            Command::Key { code, down } => {
                if !(1..=767).contains(&code) {
                    return Err("invalid evdev keycode");
                }
                if down {
                    if !self.keys.insert(code) {
                        return Err("key already down");
                    }
                } else if !self.keys.remove(&code) {
                    return Err("key is not down");
                }
            }
            Command::Action { ref name }
                if !["back", "home", "overview"].contains(&name.as_str()) =>
            {
                return Err("unknown action");
            }
            _ => {}
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn touch_transitions_and_bounds() {
        let mut i = InputState::default();
        assert!(
            i.validate(
                &Command::TouchDown {
                    slot: 0,
                    x: f64::NAN,
                    y: 0.0
                },
                400.0,
                800.0
            )
            .is_err()
        );
        assert!(
            i.validate(
                &Command::TouchMove {
                    slot: 0,
                    x: 1.0,
                    y: 1.0
                },
                400.0,
                800.0
            )
            .is_err()
        );
        let down = Command::TouchDown {
            slot: 0,
            x: 1.0,
            y: 1.0,
        };
        assert!(i.validate(&down, 400.0, 800.0).is_ok());
        assert!(i.validate(&down, 400.0, 800.0).is_err());
        assert!(
            i.validate(&Command::TouchUp { slot: 0 }, 400.0, 800.0)
                .is_ok()
        );
        assert!(
            i.validate(&Command::TouchUp { slot: 0 }, 400.0, 800.0)
                .is_err()
        );
    }
    #[test]
    fn key_transitions() {
        let mut i = InputState::default();
        assert!(
            i.validate(
                &Command::Key {
                    code: 30,
                    down: false
                },
                1.0,
                1.0
            )
            .is_err()
        );
        assert!(
            i.validate(
                &Command::Key {
                    code: 30,
                    down: true
                },
                1.0,
                1.0
            )
            .is_ok()
        );
        assert!(
            i.validate(
                &Command::Key {
                    code: 30,
                    down: true
                },
                1.0,
                1.0
            )
            .is_err()
        );
    }
}
