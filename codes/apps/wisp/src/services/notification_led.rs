//! Chassis Notification & Charging Status LED Service (platform-agnostic).
//!
//! Drives dedicated status/notification LEDs (e.g. Poco F1's bottom chin `white:status`)
//! for power cable charging indication, full-battery glow, and unread notification breathing,
//! completely separate from the rear camera flashlight ([`super::flashlight`]).

/// Platform-agnostic visual mode for the notification / charging LED.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NotificationLedMode {
    /// LED off (unplugged and no unread notifications).
    #[default]
    Off,
    /// Steady glow while external power is connected and battery is charging (`< 99%`).
    Charging,
    /// Calm low-intensity steady glow when external power is connected and battery is full (`>= 99%`).
    Charged,
    /// Smooth breathing pulse for unread notifications (keeps a subtle floor when `charging == true`).
    NotificationPulse { charging: bool },
}

/// Platform-agnostic backend contract for a notification / status LED driver.
pub trait NotificationLedBackend: Send {
    fn is_available(&self) -> bool;
    fn backend_label(&self) -> &str;
    fn set_mode(&mut self, mode: NotificationLedMode);
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullNotificationLedBackend;

impl NotificationLedBackend for NullNotificationLedBackend {
    fn is_available(&self) -> bool {
        false
    }

    fn backend_label(&self) -> &str {
        "none"
    }

    fn set_mode(&mut self, _mode: NotificationLedMode) {}
}

/// Pure policy mapping charging + unread notifications + screen/DND state onto [`NotificationLedMode`].
pub fn compute_notification_led_mode(
    plugged_in: bool,
    battery_level: u8,
    has_unread_notifs: bool,
    screen_on: bool,
    locked: bool,
    dnd_enabled: bool,
) -> NotificationLedMode {
    if has_unread_notifs && (!screen_on || locked) && !dnd_enabled {
        NotificationLedMode::NotificationPulse {
            charging: plugged_in,
        }
    } else if plugged_in {
        if battery_level >= 99 {
            NotificationLedMode::Charged
        } else {
            NotificationLedMode::Charging
        }
    } else {
        NotificationLedMode::Off
    }
}

/// Domain service coordinating notification and charging LED transitions.
pub struct NotificationLedService {
    backend: Box<dyn NotificationLedBackend>,
    current_mode: NotificationLedMode,
}

impl NotificationLedService {
    pub fn new(
        mut backend: Box<dyn NotificationLedBackend>,
        plugged_in: bool,
        battery_level: u8,
        dnd_enabled: bool,
    ) -> Self {
        let initial = compute_notification_led_mode(
            plugged_in,
            battery_level,
            false,
            true,
            false,
            dnd_enabled,
        );
        backend.set_mode(initial);
        Self {
            backend,
            current_mode: initial,
        }
    }

    pub fn is_available(&self) -> bool {
        self.backend.is_available()
    }

    pub fn backend_label(&self) -> &str {
        self.backend.backend_label()
    }

    pub fn current_mode(&self) -> NotificationLedMode {
        self.current_mode
    }

    pub fn update(
        &mut self,
        plugged_in: bool,
        battery_level: u8,
        has_unread_notifs: bool,
        screen_on: bool,
        locked: bool,
        dnd_enabled: bool,
    ) {
        let desired = compute_notification_led_mode(
            plugged_in,
            battery_level,
            has_unread_notifs,
            screen_on,
            locked,
            dnd_enabled,
        );
        if desired != self.current_mode {
            self.current_mode = desired;
            self.backend.set_mode(desired);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_handles_charging_full_unread_and_dnd() {
        assert_eq!(
            compute_notification_led_mode(false, 50, false, true, false, false),
            NotificationLedMode::Off
        );
        assert_eq!(
            compute_notification_led_mode(true, 62, false, true, false, false),
            NotificationLedMode::Charging
        );
        assert_eq!(
            compute_notification_led_mode(true, 99, false, false, true, false),
            NotificationLedMode::Charged
        );
        assert_eq!(
            compute_notification_led_mode(true, 70, true, false, true, false),
            NotificationLedMode::NotificationPulse { charging: true }
        );
        assert_eq!(
            compute_notification_led_mode(true, 70, true, false, true, true),
            NotificationLedMode::Charging
        );
    }
}
