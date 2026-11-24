//! Freedesktop.org Desktop Notifications Specification v1.3 implementation for Wisp.
//!
//! Reference: `docs/freedesktop/notification/index.md`
//! - Session D-Bus service name: `org.freedesktop.Notifications`
//! - Object path & interface:    `/org/freedesktop/Notifications` (`org.freedesktop.Notifications`)
//! - Supported methods:          `GetCapabilities`, `GetServerInformation`, `Notify`, `CloseNotification`,
//!                               plus `org.freedesktop.DBus.Introspectable.Introspect`
//! - Emitted signals:            `NotificationClosed(uu)`, `ActivationToken(us)`, `ActionInvoked(us)`
//! - Supported capabilities:     `actions`, `body`, `body-markup`, `icon-static`, `persistence`
//! - Supported hints:            `urgency`, `category`, `desktop-entry`, `resident`, `transient`,
//!                               `suppress-sound`, `image-data`/`image_data`/`icon_data` (`(iiibiiay)`),
//!                               `image-path`/`image_path`
//!
//! Uses `libsystemd`'s non-variadic `sd-bus` C ABI (`#[cfg(feature = "compositor")]`) directly
//! inside Smithay's `calloop` event loop with zero extra crate dependencies.

use bhoo::Image;
use spectre_settings::NotificationHistoryEntry;

/// Standard Freedesktop `NotificationClosed` reason codes (§9.2.1 Table 8).
pub const CLOSE_REASON_EXPIRED: u32 = 1;
pub const CLOSE_REASON_DISMISSED: u32 = 2;
pub const CLOSE_REASON_CLOSED_CALL: u32 = 3;
#[allow(dead_code)]
pub const CLOSE_REASON_UNDEFINED: u32 = 4;

/// Raw `(iiibiiay)` image payload per Desktop Notifications Spec §5.2 (`image-data` / `icon_data`).
#[derive(Clone, Debug)]
pub struct RawImageData {
    pub width: u32,
    pub height: u32,
    pub rowstride: usize,
    pub has_alpha: bool,
    pub bits_per_sample: i32,
    pub channels: i32,
    pub data: Vec<u8>,
}

impl RawImageData {
    /// Convert Spec §5.2 RGB or RGBA raw image data into a GPU-ready `bhoo::Image` (capped to 128px).
    pub fn to_bhoo_image(&self) -> Option<Image> {
        if self.width == 0
            || self.height == 0
            || self.width > 2048
            || self.height > 2048
            || self.bits_per_sample != 8
        {
            return None;
        }
        let ch = if self.has_alpha { 4usize } else { 3usize };
        if self.channels as usize != ch {
            return None;
        }
        let w = self.width as usize;
        let h = self.height as usize;
        let min_stride = w.checked_mul(ch)?;
        if self.rowstride < min_stride {
            return None;
        }
        let needed = (h - 1).checked_mul(self.rowstride)?.checked_add(min_stride)?;
        if self.data.len() < needed {
            return None;
        }

        // Downscale oversized bitmaps safely to <= 128px square so the GPU texture atlas stays light.
        let max_dim = w.max(h);
        let (out_w, out_h) = if max_dim > 128 {
            let scale = 128.0 / max_dim as f32;
            (
                ((w as f32 * scale).round() as usize).max(1),
                ((h as f32 * scale).round() as usize).max(1),
            )
        } else {
            (w, h)
        };

        let mut rgba = vec![0u8; out_w * out_h * 4];
        for oy in 0..out_h {
            let sy = (oy * h / out_h).min(h - 1);
            let row_off = sy * self.rowstride;
            for ox in 0..out_w {
                let sx = (ox * w / out_w).min(w - 1);
                let src = row_off + sx * ch;
                let dst = (oy * out_w + ox) * 4;
                rgba[dst] = self.data[src];
                rgba[dst + 1] = self.data[src + 1];
                rgba[dst + 2] = self.data[src + 2];
                rgba[dst + 3] = if self.has_alpha { self.data[src + 3] } else { 255 };
            }
        }
        Image::rgba(out_w as u32, out_h as u32, rgba)
    }
}

/// Parsed incoming `org.freedesktop.Notifications.Notify` payload.
#[derive(Clone, Debug)]
pub struct IncomingNotification {
    pub id: u32,
    pub replaces_id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<(String, String)>,
    pub urgency: u8,
    pub category: Option<String>,
    pub desktop_entry: Option<String>,
    pub resident: bool,
    pub transient: bool,
    pub suppress_sound: bool,
    pub image_data: Option<RawImageData>,
    pub image_path: Option<String>,
    pub icon_data: Option<RawImageData>,
    pub expire_timeout_ms: i32,
}

/// Events produced by the D-Bus notification server for `WispShell`.
#[derive(Clone, Debug)]
pub enum NotificationEvent {
    Notify(IncomingNotification),
    ClosedByClient(u32),
}

/// Strip Freedesktop v1.3 §4 XML markup tags (`<b>`, `<i>`, `<u>`, `<a href>`, `<img alt>`)
/// and decode XML character entities into clean display UTF-8 text.
pub fn strip_notification_markup(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<'
            && let Some(rel_end) = input[i..].find('>')
        {
            let tag = &input[i + 1..i + rel_end];
            let trimmed = tag.trim();
            let lower = trimmed.to_ascii_lowercase();
            // Extract alt="..." for <img ...> per Spec §4.2
            if lower.starts_with("img")
                && let Some(alt_idx) = lower.find("alt=")
            {
                let rest = &trimmed[alt_idx + 4..];
                if let Some(q) = rest.chars().next()
                    && (q == '"' || q == '\'')
                    && let Some(end_q) = rest[1..].find(q)
                {
                    out.push_str(&rest[1..1 + end_q]);
                }
            } else if lower.starts_with("br") {
                out.push('\n');
            }
            i += rel_end + 1;
            continue;
        }
        if bytes[i] == b'&'
            && let Some(rel_semi) = input[i..].find(';')
            && rel_semi <= 10
        {
            let entity = &input[i + 1..i + rel_semi];
            let decoded = match entity {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some(' '),
                _ => {
                    if let Some(hex) = entity.strip_prefix("#x").or_else(|| entity.strip_prefix("#X")) {
                        u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                    } else if let Some(dec) = entity.strip_prefix('#') {
                        dec.parse::<u32>().ok().and_then(char::from_u32)
                    } else {
                        None
                    }
                }
            };
            if let Some(ch) = decoded {
                out.push(ch);
                i += rel_semi + 1;
                continue;
            }
        }
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Resolve a Freedesktop `file://` URI or local path into a `PathBuf` (Spec §5.2).
pub fn resolve_icon_uri_or_path(spec: &str) -> Option<std::path::PathBuf> {
    let s = spec.trim();
    if s.is_empty() {
        return None;
    }
    let path_str = if let Some(rest) = s.strip_prefix("file://") {
        // Basic percent-decoding for spaces (`%20`)
        let decoded = rest.replace("%20", " ");
        std::path::PathBuf::from(decoded)
    } else if s.starts_with('/') {
        std::path::PathBuf::from(s)
    } else {
        return None;
    };
    path_str.is_file().then_some(path_str)
}

/// Persist notification history cleanly to `~/.config/wisp/notifications-history.tsv`
/// so both embedded Settings and standalone `spectre-settings` share the exact same log.
pub fn save_notification_history(entries: &[NotificationHistoryEntry]) {
    #[cfg(all(not(target_arch = "wasm32"), not(test)))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let dir = std::path::PathBuf::from(home).join(".config/wisp");
            let _ = std::fs::create_dir_all(&dir);
            let mut out = String::new();
            for e in entries.iter().take(80) {
                let clean = |s: &str| s.replace(['\t', '\r', '\n'], " ");
                out.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    e.id,
                    clean(&e.app),
                    clean(&e.title),
                    clean(&e.body),
                    clean(&e.time_label),
                    if e.dismissed { 1 } else { 0 },
                    e.urgency,
                ));
            }
            let path = dir.join("notifications-history.tsv");
            let tmp = dir.join("notifications-history.tsv.tmp");
            if std::fs::write(&tmp, out).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }
    #[cfg(any(target_arch = "wasm32", test))]
    {
        let _ = entries;
    }
}

/// Load notification history from `~/.config/wisp/notifications-history.tsv`.
pub fn load_notification_history() -> Vec<NotificationHistoryEntry> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(home) = std::env::var_os("HOME") else {
            return Vec::new();
        };
        let path = std::path::PathBuf::from(home).join(".config/wisp/notifications-history.tsv");
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for line in text.lines() {
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 7 {
                continue;
            }
            out.push(NotificationHistoryEntry {
                id: cols[0].parse().unwrap_or(0),
                app: cols[1].to_string(),
                title: cols[2].to_string(),
                body: cols[3].to_string(),
                time_label: cols[4].to_string(),
                dismissed: cols[5] == "1",
                urgency: cols[6].parse().unwrap_or(1),
            });
        }
        out
    }
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }
}

// =============================================================================
// NATIVE D-BUS NOTIFICATION SERVER (`libsystemd` `sd-bus` C ABI)
// =============================================================================

#[cfg(feature = "compositor")]
mod dbus_server {
    use super::*;
    use std::collections::{HashSet, VecDeque};
    use std::ffi::{CStr, CString, c_char, c_int, c_void};
    use std::os::unix::io::RawFd;
    use std::ptr;

    #[repr(C)]
    struct SdBus {
        _opaque: [u8; 0],
    }
    #[repr(C)]
    struct SdBusSlot {
        _opaque: [u8; 0],
    }
    #[repr(C)]
    struct SdBusMessage {
        _opaque: [u8; 0],
    }
    #[repr(C)]
    struct SdBusError {
        name: *const c_char,
        message: *const c_char,
        _need_free: c_int,
    }

    type SdBusMessageHandler =
        unsafe extern "C" fn(*mut SdBusMessage, *mut c_void, *mut SdBusError) -> c_int;

    const SD_BUS_NAME_REPLACE_EXISTING: u64 = 1 << 0;
    const SD_BUS_NAME_ALLOW_REPLACEMENT: u64 = 1 << 1;

    #[link(name = "systemd")]
    unsafe extern "C" {
        fn sd_bus_default_user(ret: *mut *mut SdBus) -> c_int;
        fn sd_bus_flush_close_unref(bus: *mut SdBus) -> *mut SdBus;
        fn sd_bus_request_name(bus: *mut SdBus, name: *const c_char, flags: u64) -> c_int;
        fn sd_bus_add_object(
            bus: *mut SdBus,
            ret_slot: *mut *mut SdBusSlot,
            path: *const c_char,
            callback: SdBusMessageHandler,
            userdata: *mut c_void,
        ) -> c_int;
        fn sd_bus_slot_unref(slot: *mut SdBusSlot) -> *mut SdBusSlot;
        fn sd_bus_get_fd(bus: *mut SdBus) -> c_int;
        fn sd_bus_process(bus: *mut SdBus, ret: *mut *mut SdBusMessage) -> c_int;
        fn sd_bus_flush(bus: *mut SdBus) -> c_int;
        fn sd_bus_send(
            bus: *mut SdBus,
            m: *mut SdBusMessage,
            ret_cookie: *mut u64,
        ) -> c_int;
        fn sd_bus_message_unref(m: *mut SdBusMessage) -> *mut SdBusMessage;
        fn sd_bus_message_is_method_call(
            m: *mut SdBusMessage,
            interface: *const c_char,
            member: *const c_char,
        ) -> c_int;
        fn sd_bus_message_new_method_return(
            call: *mut SdBusMessage,
            ret: *mut *mut SdBusMessage,
        ) -> c_int;
        fn sd_bus_message_new_method_error(
            call: *mut SdBusMessage,
            ret: *mut *mut SdBusMessage,
            e: *const SdBusError,
        ) -> c_int;
        fn sd_bus_message_new_signal(
            bus: *mut SdBus,
            ret: *mut *mut SdBusMessage,
            path: *const c_char,
            interface: *const c_char,
            member: *const c_char,
        ) -> c_int;
        fn sd_bus_message_new_method_call(
            bus: *mut SdBus,
            ret: *mut *mut SdBusMessage,
            destination: *const c_char,
            path: *const c_char,
            interface: *const c_char,
            member: *const c_char,
        ) -> c_int;
        fn sd_bus_message_set_expect_reply(m: *mut SdBusMessage, b: c_int) -> c_int;
        fn sd_bus_message_open_container(
            m: *mut SdBusMessage,
            container_type: c_char,
            contents: *const c_char,
        ) -> c_int;
        fn sd_bus_message_close_container(m: *mut SdBusMessage) -> c_int;
        fn sd_bus_message_append_basic(
            m: *mut SdBusMessage,
            value_type: c_char,
            p: *const c_void,
        ) -> c_int;
        fn sd_bus_message_enter_container(
            m: *mut SdBusMessage,
            container_type: c_char,
            contents: *const c_char,
        ) -> c_int;
        fn sd_bus_message_exit_container(m: *mut SdBusMessage) -> c_int;
        fn sd_bus_message_peek_type(
            m: *mut SdBusMessage,
            ret_type: *mut c_char,
            ret_contents: *mut *const c_char,
        ) -> c_int;
        fn sd_bus_message_read_basic(
            m: *mut SdBusMessage,
            value_type: c_char,
            ret: *mut c_void,
        ) -> c_int;
        fn sd_bus_message_read_array(
            m: *mut SdBusMessage,
            element_type: c_char,
            ret_ptr: *mut *const c_void,
            ret_size: *mut usize,
        ) -> c_int;
        fn sd_bus_message_skip(m: *mut SdBusMessage, types: *const c_char) -> c_int;
    }

    const INTROSPECT_XML: &str = r#"<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node name="/org/freedesktop/Notifications">
  <interface name="org.freedesktop.DBus.Introspectable">
    <method name="Introspect">
      <arg name="xml_data" type="s" direction="out"/>
    </method>
  </interface>
  <interface name="org.freedesktop.Notifications">
    <method name="GetCapabilities">
      <arg name="capabilities" type="as" direction="out"/>
    </method>
    <method name="Notify">
      <arg name="app_name" type="s" direction="in"/>
      <arg name="replaces_id" type="u" direction="in"/>
      <arg name="app_icon" type="s" direction="in"/>
      <arg name="summary" type="s" direction="in"/>
      <arg name="body" type="s" direction="in"/>
      <arg name="actions" type="as" direction="in"/>
      <arg name="hints" type="a{sv}" direction="in"/>
      <arg name="expire_timeout" type="i" direction="in"/>
      <arg name="id" type="u" direction="out"/>
    </method>
    <method name="CloseNotification">
      <arg name="id" type="u" direction="in"/>
    </method>
    <method name="GetServerInformation">
      <arg name="name" type="s" direction="out"/>
      <arg name="vendor" type="s" direction="out"/>
      <arg name="version" type="s" direction="out"/>
      <arg name="spec_version" type="s" direction="out"/>
    </method>
    <signal name="NotificationClosed">
      <arg name="id" type="u"/>
      <arg name="reason" type="u"/>
    </signal>
    <signal name="ActionInvoked">
      <arg name="id" type="u"/>
      <arg name="action_key" type="s"/>
    </signal>
    <signal name="ActivationToken">
      <arg name="id" type="u"/>
      <arg name="activation_token" type="s"/>
    </signal>
  </interface>
</node>
"#;

    struct SharedState {
        bus: *mut SdBus,
        next_id: u32,
        active_ids: HashSet<u32>,
        events: VecDeque<NotificationEvent>,
    }

    impl SharedState {
        fn alloc_id(&mut self, replaces_id: u32) -> u32 {
            if replaces_id != 0 {
                self.active_ids.insert(replaces_id);
                return replaces_id;
            }
            let id = self.next_id.max(1);
            self.next_id = self.next_id.wrapping_add(1);
            if self.next_id == 0 {
                self.next_id = 1;
            }
            self.active_ids.insert(id);
            id
        }
    }

    unsafe fn read_string(m: *mut SdBusMessage) -> Option<String> {
        let mut p: *const c_char = ptr::null();
        let r = unsafe { sd_bus_message_read_basic(m, b's' as c_char, (&mut p as *mut *const c_char).cast()) };
        if r <= 0 {
            return None;
        }
        if p.is_null() {
            return Some(String::new());
        }
        Some(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
    }

    unsafe fn read_u32(m: *mut SdBusMessage) -> Option<u32> {
        let mut v: u32 = 0;
        let r = unsafe { sd_bus_message_read_basic(m, b'u' as c_char, (&mut v as *mut u32).cast()) };
        (r > 0).then_some(v)
    }

    unsafe fn read_i32(m: *mut SdBusMessage) -> Option<i32> {
        let mut v: i32 = 0;
        let r = unsafe { sd_bus_message_read_basic(m, b'i' as c_char, (&mut v as *mut i32).cast()) };
        (r > 0).then_some(v)
    }

    unsafe fn read_u8(m: *mut SdBusMessage) -> Option<u8> {
        let mut v: u8 = 0;
        let r = unsafe { sd_bus_message_read_basic(m, b'y' as c_char, (&mut v as *mut u8).cast()) };
        (r > 0).then_some(v)
    }

    unsafe fn read_bool(m: *mut SdBusMessage) -> Option<bool> {
        let mut v: c_int = 0;
        let r = unsafe { sd_bus_message_read_basic(m, b'b' as c_char, (&mut v as *mut c_int).cast()) };
        (r > 0).then_some(v != 0)
    }

    unsafe fn read_raw_image_struct(m: *mut SdBusMessage) -> Option<RawImageData> {
        let sig = c"iiibiiay";
        if unsafe { sd_bus_message_enter_container(m, b'r' as c_char, sig.as_ptr()) } <= 0 {
            return None;
        }
        let width = unsafe { read_i32(m) };
        let height = unsafe { read_i32(m) };
        let rowstride = unsafe { read_i32(m) };
        let has_alpha = unsafe { read_bool(m) };
        let bps = unsafe { read_i32(m) };
        let channels = unsafe { read_i32(m) };
        let mut data_ptr: *const c_void = ptr::null();
        let mut data_len: usize = 0;
        let arr_r = unsafe {
            sd_bus_message_read_array(m, b'y' as c_char, &mut data_ptr, &mut data_len)
        };
        unsafe { sd_bus_message_exit_container(m) };

        if arr_r < 0 || data_len > 16 * 1024 * 1024 {
            return None;
        }
        let (Some(w), Some(h), Some(rs), Some(ha), Some(b), Some(ch)) =
            (width, height, rowstride, has_alpha, bps, channels)
        else {
            return None;
        };
        if w <= 0 || h <= 0 || rs <= 0 {
            return None;
        }
        let data = if data_len > 0 && !data_ptr.is_null() {
            unsafe { std::slice::from_raw_parts(data_ptr.cast::<u8>(), data_len) }.to_vec()
        } else {
            Vec::new()
        };
        Some(RawImageData {
            width: w as u32,
            height: h as u32,
            rowstride: rs as usize,
            has_alpha: ha,
            bits_per_sample: b,
            channels: ch,
            data,
        })
    }

    unsafe fn send_empty_return(bus: *mut SdBus, call: *mut SdBusMessage) {
        let mut reply: *mut SdBusMessage = ptr::null_mut();
        if unsafe { sd_bus_message_new_method_return(call, &mut reply) } >= 0 && !reply.is_null() {
            unsafe {
                sd_bus_send(bus, reply, ptr::null_mut());
                sd_bus_message_unref(reply);
            }
        }
    }

    unsafe fn send_string_return(bus: *mut SdBus, call: *mut SdBusMessage, value: &str) {
        let mut reply: *mut SdBusMessage = ptr::null_mut();
        if unsafe { sd_bus_message_new_method_return(call, &mut reply) } >= 0 && !reply.is_null() {
            if let Ok(cval) = CString::new(value) {
                unsafe {
                    sd_bus_message_append_basic(reply, b's' as c_char, cval.as_ptr().cast());
                    sd_bus_send(bus, reply, ptr::null_mut());
                }
            }
            unsafe { sd_bus_message_unref(reply) };
        }
    }

    unsafe fn send_u32_return(bus: *mut SdBus, call: *mut SdBusMessage, value: u32) {
        let mut reply: *mut SdBusMessage = ptr::null_mut();
        if unsafe { sd_bus_message_new_method_return(call, &mut reply) } >= 0 && !reply.is_null() {
            unsafe {
                sd_bus_message_append_basic(reply, b'u' as c_char, (&value as *const u32).cast());
                sd_bus_send(bus, reply, ptr::null_mut());
                sd_bus_message_unref(reply);
            }
        }
    }

    unsafe fn send_error(bus: *mut SdBus, call: *mut SdBusMessage, name: &CStr, msg: &CStr) {
        let err = SdBusError {
            name: name.as_ptr(),
            message: msg.as_ptr(),
            _need_free: 0,
        };
        let mut reply: *mut SdBusMessage = ptr::null_mut();
        if unsafe { sd_bus_message_new_method_error(call, &mut reply, &err) } >= 0 && !reply.is_null() {
            unsafe {
                sd_bus_send(bus, reply, ptr::null_mut());
                sd_bus_message_unref(reply);
            }
        }
    }

    unsafe fn emit_closed_raw(bus: *mut SdBus, id: u32, reason: u32) {
        let mut sig: *mut SdBusMessage = ptr::null_mut();
        if unsafe {
            sd_bus_message_new_signal(
                bus,
                &mut sig,
                c"/org/freedesktop/Notifications".as_ptr(),
                c"org.freedesktop.Notifications".as_ptr(),
                c"NotificationClosed".as_ptr(),
            )
        } >= 0
            && !sig.is_null()
        {
            unsafe {
                sd_bus_message_append_basic(sig, b'u' as c_char, (&id as *const u32).cast());
                sd_bus_message_append_basic(sig, b'u' as c_char, (&reason as *const u32).cast());
                sd_bus_send(bus, sig, ptr::null_mut());
                sd_bus_message_unref(sig);
            }
        }
    }

    unsafe fn emit_u32_str_signal(bus: *mut SdBus, member: &CStr, id: u32, text: &str) {
        let Ok(ctext) = CString::new(text) else {
            return;
        };
        let mut sig: *mut SdBusMessage = ptr::null_mut();
        if unsafe {
            sd_bus_message_new_signal(
                bus,
                &mut sig,
                c"/org/freedesktop/Notifications".as_ptr(),
                c"org.freedesktop.Notifications".as_ptr(),
                member.as_ptr(),
            )
        } >= 0
            && !sig.is_null()
        {
            unsafe {
                sd_bus_message_append_basic(sig, b'u' as c_char, (&id as *const u32).cast());
                sd_bus_message_append_basic(sig, b's' as c_char, ctext.as_ptr().cast());
                sd_bus_send(bus, sig, ptr::null_mut());
                sd_bus_message_unref(sig);
            }
        }
    }

    unsafe extern "C" fn handle_notifications_message(
        m: *mut SdBusMessage,
        userdata: *mut c_void,
        _ret_error: *mut SdBusError,
    ) -> c_int {
        if m.is_null() || userdata.is_null() {
            return 0;
        }
        let state = unsafe { &mut *userdata.cast::<SharedState>() };
        let bus = state.bus;

        // 1. org.freedesktop.DBus.Introspectable.Introspect
        if unsafe {
            sd_bus_message_is_method_call(
                m,
                c"org.freedesktop.DBus.Introspectable".as_ptr(),
                c"Introspect".as_ptr(),
            )
        } > 0
        {
            unsafe { send_string_return(bus, m, INTROSPECT_XML) };
            return 1;
        }

        // 2. org.freedesktop.Notifications.GetCapabilities() -> as
        if unsafe {
            sd_bus_message_is_method_call(
                m,
                c"org.freedesktop.Notifications".as_ptr(),
                c"GetCapabilities".as_ptr(),
            )
        } > 0
        {
            let mut reply: *mut SdBusMessage = ptr::null_mut();
            if unsafe { sd_bus_message_new_method_return(m, &mut reply) } >= 0 && !reply.is_null() {
                unsafe {
                    if sd_bus_message_open_container(reply, b'a' as c_char, c"s".as_ptr()) >= 0 {
                        for cap in [
                            c"actions",
                            c"body",
                            c"body-markup",
                            c"icon-static",
                            c"persistence",
                        ] {
                            sd_bus_message_append_basic(reply, b's' as c_char, cap.as_ptr().cast());
                        }
                        sd_bus_message_close_container(reply);
                    }
                    sd_bus_send(bus, reply, ptr::null_mut());
                    sd_bus_message_unref(reply);
                }
            }
            return 1;
        }

        // 3. org.freedesktop.Notifications.GetServerInformation() -> (ssss)
        if unsafe {
            sd_bus_message_is_method_call(
                m,
                c"org.freedesktop.Notifications".as_ptr(),
                c"GetServerInformation".as_ptr(),
            )
        } > 0
        {
            let mut reply: *mut SdBusMessage = ptr::null_mut();
            if unsafe { sd_bus_message_new_method_return(m, &mut reply) } >= 0 && !reply.is_null() {
                unsafe {
                    for s in [c"Wisp", c"SpectreOS", c"0.1.0", c"1.3"] {
                        sd_bus_message_append_basic(reply, b's' as c_char, s.as_ptr().cast());
                    }
                    sd_bus_send(bus, reply, ptr::null_mut());
                    sd_bus_message_unref(reply);
                }
            }
            return 1;
        }

        // 4. org.freedesktop.Notifications.CloseNotification(u)
        if unsafe {
            sd_bus_message_is_method_call(
                m,
                c"org.freedesktop.Notifications".as_ptr(),
                c"CloseNotification".as_ptr(),
            )
        } > 0
        {
            let Some(id) = (unsafe { read_u32(m) }) else {
                unsafe {
                    send_error(
                        bus,
                        m,
                        c"org.freedesktop.DBus.Error.InvalidArgs",
                        c"Expected uint32 notification id",
                    );
                }
                return 1;
            };
            if state.active_ids.remove(&id) {
                unsafe {
                    emit_closed_raw(bus, id, CLOSE_REASON_CLOSED_CALL);
                    send_empty_return(bus, m);
                }
                state.events.push_back(NotificationEvent::ClosedByClient(id));
            } else {
                // Spec §9.1.3: "If the notification no longer exists, an empty D-Bus Error message is sent back."
                unsafe {
                    send_error(bus, m, c"org.freedesktop.Notifications.Error.InvalidId", c"");
                }
            }
            return 1;
        }

        // 5. org.freedesktop.Notifications.Notify(susssasa{sv}i) -> u
        if unsafe {
            sd_bus_message_is_method_call(
                m,
                c"org.freedesktop.Notifications".as_ptr(),
                c"Notify".as_ptr(),
            )
        } > 0
        {
            let Some(app_name) = (unsafe { read_string(m) }) else {
                unsafe {
                    send_error(bus, m, c"org.freedesktop.DBus.Error.InvalidArgs", c"Bad app_name");
                }
                return 1;
            };
            let Some(replaces_id) = (unsafe { read_u32(m) }) else {
                unsafe {
                    send_error(bus, m, c"org.freedesktop.DBus.Error.InvalidArgs", c"Bad replaces_id");
                }
                return 1;
            };
            let Some(app_icon) = (unsafe { read_string(m) }) else {
                unsafe {
                    send_error(bus, m, c"org.freedesktop.DBus.Error.InvalidArgs", c"Bad app_icon");
                }
                return 1;
            };
            let Some(summary) = (unsafe { read_string(m) }) else {
                unsafe {
                    send_error(bus, m, c"org.freedesktop.DBus.Error.InvalidArgs", c"Bad summary");
                }
                return 1;
            };
            let Some(body_raw) = (unsafe { read_string(m) }) else {
                unsafe {
                    send_error(bus, m, c"org.freedesktop.DBus.Error.InvalidArgs", c"Bad body");
                }
                return 1;
            };

            // Read `actions` (`as` -> pairs of (key, localized_label))
            let mut actions = Vec::new();
            if unsafe { sd_bus_message_enter_container(m, b'a' as c_char, c"s".as_ptr()) } >= 0 {
                let mut flat = Vec::new();
                while let Some(s) = unsafe { read_string(m) } {
                    flat.push(s);
                }
                unsafe { sd_bus_message_exit_container(m) };
                for pair in flat.chunks_exact(2) {
                    actions.push((pair[0].clone(), pair[1].clone()));
                }
            }

            // Read `hints` (`a{sv}`)
            let mut urgency: u8 = 1; // Default Normal per Spec §7
            let mut category: Option<String> = None;
            let mut desktop_entry: Option<String> = None;
            let mut resident = false;
            let mut transient = false;
            let mut suppress_sound = false;
            let mut image_data: Option<RawImageData> = None;
            let mut image_path: Option<String> = None;
            let mut icon_data: Option<RawImageData> = None;

            if unsafe { sd_bus_message_enter_container(m, b'a' as c_char, c"{sv}".as_ptr()) } >= 0 {
                while unsafe { sd_bus_message_enter_container(m, b'e' as c_char, c"sv".as_ptr()) } > 0 {
                    let key = unsafe { read_string(m) }.unwrap_or_default();
                    let mut vtype: c_char = 0;
                    let mut contents: *const c_char = ptr::null();
                    if unsafe { sd_bus_message_peek_type(m, &mut vtype, &mut contents) } > 0
                        && vtype == b'v' as c_char
                        && !contents.is_null()
                    {
                        let sig = unsafe { CStr::from_ptr(contents) }.to_str().unwrap_or("");
                        if unsafe { sd_bus_message_enter_container(m, b'v' as c_char, contents) } > 0 {
                            match (key.as_str(), sig) {
                                ("urgency", "y") => {
                                    if let Some(u) = unsafe { read_u8(m) } {
                                        urgency = u.min(2);
                                    }
                                }
                                ("urgency", "u") => {
                                    if let Some(u) = unsafe { read_u32(m) } {
                                        urgency = (u as u8).min(2);
                                    }
                                }
                                ("urgency", "i") => {
                                    if let Some(u) = unsafe { read_i32(m) } {
                                        urgency = u.clamp(0, 2) as u8;
                                    }
                                }
                                ("category", "s") => category = unsafe { read_string(m) },
                                ("desktop-entry", "s") => desktop_entry = unsafe { read_string(m) },
                                ("resident", "b") => {
                                    if let Some(b) = unsafe { read_bool(m) } {
                                        resident = b;
                                    }
                                }
                                ("transient", "b") => {
                                    if let Some(b) = unsafe { read_bool(m) } {
                                        transient = b;
                                    }
                                }
                                ("suppress-sound", "b") => {
                                    if let Some(b) = unsafe { read_bool(m) } {
                                        suppress_sound = b;
                                    }
                                }
                                ("image-path" | "image_path", "s") => {
                                    image_path = unsafe { read_string(m) };
                                }
                                ("image-data" | "image_data", "(iiibiiay)") => {
                                    image_data = unsafe { read_raw_image_struct(m) };
                                }
                                ("icon_data", "(iiibiiay)") => {
                                    icon_data = unsafe { read_raw_image_struct(m) };
                                }
                                _ => {
                                    unsafe { sd_bus_message_skip(m, contents) };
                                }
                            }
                            unsafe { sd_bus_message_exit_container(m) };
                        }
                    } else {
                        unsafe { sd_bus_message_skip(m, c"v".as_ptr()) };
                    }
                    unsafe { sd_bus_message_exit_container(m) };
                }
                unsafe { sd_bus_message_exit_container(m) };
            }

            let expire_timeout_ms = unsafe { read_i32(m) }.unwrap_or(-1);
            let id = state.alloc_id(replaces_id);
            let body = strip_notification_markup(&body_raw);
            let app_label = if app_name.trim().is_empty() {
                desktop_entry
                    .clone()
                    .unwrap_or_else(|| "System".to_string())
            } else {
                app_name
            };

            tracing::info!(
                "Freedesktop Notify id={id} replaces={replaces_id} app='{app_label}' urgency={urgency} summary='{}'",
                summary.chars().take(60).collect::<String>()
            );

            state.events.push_back(NotificationEvent::Notify(IncomingNotification {
                id,
                replaces_id,
                app_name: app_label,
                app_icon,
                summary,
                body,
                actions,
                urgency,
                category,
                desktop_entry,
                resident,
                transient,
                suppress_sound,
                image_data,
                image_path,
                icon_data,
                expire_timeout_ms,
            }));

            unsafe { send_u32_return(bus, m, id) };
            return 1;
        }

        0
    }

    /// Live `org.freedesktop.Notifications` D-Bus service owned by Wisp's compositor loop.
    pub struct NotificationDaemon {
        bus: *mut SdBus,
        slot: *mut SdBusSlot,
        shared: Box<SharedState>,
        fd: RawFd,
    }

    impl NotificationDaemon {
        /// Connect to the session bus and claim `org.freedesktop.Notifications` at
        /// `/org/freedesktop/Notifications`.
        pub fn start() -> Result<Self, String> {
            let mut bus: *mut SdBus = ptr::null_mut();
            let r = unsafe { sd_bus_default_user(&mut bus) };
            if r < 0 || bus.is_null() {
                return Err(format!("sd_bus_default_user failed: errno={}", -r));
            }

            let mut shared = Box::new(SharedState {
                bus,
                next_id: 100,
                active_ids: HashSet::new(),
                events: VecDeque::new(),
            });

            let mut slot: *mut SdBusSlot = ptr::null_mut();
            let r_obj = unsafe {
                sd_bus_add_object(
                    bus,
                    &mut slot,
                    c"/org/freedesktop/Notifications".as_ptr(),
                    handle_notifications_message,
                    (&mut *shared as *mut SharedState).cast(),
                )
            };
            if r_obj < 0 {
                unsafe { sd_bus_flush_close_unref(bus) };
                return Err(format!("sd_bus_add_object failed: errno={}", -r_obj));
            }

            let r_name = unsafe {
                sd_bus_request_name(
                    bus,
                    c"org.freedesktop.Notifications".as_ptr(),
                    SD_BUS_NAME_REPLACE_EXISTING | SD_BUS_NAME_ALLOW_REPLACEMENT,
                )
            };
            if r_name < 0 {
                unsafe {
                    sd_bus_slot_unref(slot);
                    sd_bus_flush_close_unref(bus);
                }
                return Err(format!(
                    "sd_bus_request_name(org.freedesktop.Notifications) failed: errno={}",
                    -r_name
                ));
            }

            let fd = unsafe { sd_bus_get_fd(bus) };
            if fd < 0 {
                unsafe {
                    sd_bus_slot_unref(slot);
                    sd_bus_flush_close_unref(bus);
                }
                return Err(format!("sd_bus_get_fd failed: errno={}", -fd));
            }

            tracing::info!(
                "Claimed D-Bus service org.freedesktop.Notifications at /org/freedesktop/Notifications (fd={fd})"
            );
            Ok(Self {
                bus,
                slot,
                shared,
                fd,
            })
        }

        pub fn fd(&self) -> RawFd {
            self.fd
        }

        /// Process all pending D-Bus messages and return newly queued notification events.
        pub fn process(&mut self) -> Vec<NotificationEvent> {
            loop {
                let r = unsafe { sd_bus_process(self.bus, ptr::null_mut()) };
                if r <= 0 {
                    break;
                }
            }
            unsafe { sd_bus_flush(self.bus) };
            self.shared.events.drain(..).collect()
        }

        /// Emit `NotificationClosed(id, reason)` and invalidate `id` (Spec §9.2.1).
        pub fn emit_closed(&mut self, id: u32, reason: u32) {
            if self.shared.active_ids.remove(&id) {
                unsafe {
                    emit_closed_raw(self.bus, id, reason);
                    sd_bus_flush(self.bus);
                }
            }
        }

        /// Emit `ActivationToken(id, token)` followed by `ActionInvoked(id, action_key)` (Spec §9.2.2–§9.2.3).
        pub fn emit_action_invoked(&mut self, id: u32, action_key: &str) {
            let token = format!("wisp-notif-{id}");
            unsafe {
                emit_u32_str_signal(self.bus, c"ActivationToken", id, &token);
                emit_u32_str_signal(self.bus, c"ActionInvoked", id, action_key);
                sd_bus_flush(self.bus);
            }
        }

        /// Send a non-blocking D-Bus `SetVisible(false)` call to SpectreOS Keyboard
        /// (`org.spectreos.OSK0`) so Back gestures dismiss D-Bus-opened keyboards too.
        pub fn dismiss_osk(&mut self) {
            let mut msg: *mut SdBusMessage = ptr::null_mut();
            let visible: c_int = 0;
            unsafe {
                if sd_bus_message_new_method_call(
                    self.bus,
                    &mut msg,
                    c"org.spectreos.OSK0".as_ptr(),
                    c"/org/spectreos/OSK0".as_ptr(),
                    c"org.spectreos.OSK0".as_ptr(),
                    c"SetVisible".as_ptr(),
                ) >= 0
                    && !msg.is_null()
                {
                    sd_bus_message_set_expect_reply(msg, 0);
                    sd_bus_message_append_basic(msg, b'b' as c_char, (&visible as *const c_int).cast());
                    sd_bus_send(self.bus, msg, ptr::null_mut());
                    sd_bus_message_unref(msg);
                    sd_bus_flush(self.bus);
                }
            }
        }
    }

    impl Drop for NotificationDaemon {
        fn drop(&mut self) {
            unsafe {
                if !self.slot.is_null() {
                    sd_bus_slot_unref(self.slot);
                }
                if !self.bus.is_null() {
                    sd_bus_flush_close_unref(self.bus);
                }
            }
        }
    }
}

#[cfg(feature = "compositor")]
pub use dbus_server::NotificationDaemon;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_freedesktop_v13_markup_and_decodes_entities() {
        let raw = r#"<b>Alice</b> sent <a href="https://example.org">a photo</a> &amp; <img src="file:///tmp/a.png" alt="[Smile]"/> &lt;3"#;
        assert_eq!(
            strip_notification_markup(raw),
            "Alice sent a photo & [Smile] <3"
        );
    }

    #[test]
    fn decodes_rgb_and_rgba_image_data_hint() {
        let rgb = RawImageData {
            width: 2,
            height: 2,
            rowstride: 8, // padded rowstride
            has_alpha: false,
            bits_per_sample: 8,
            channels: 3,
            data: vec![
                255, 0, 0, 0, 255, 0, 0, 0, // row 0 + 2 pad bytes
                0, 0, 255, 255, 255, 255, 0, 0, // row 1 + 2 pad bytes
            ],
        };
        let img = rgb.to_bhoo_image().expect("valid RGB image-data");
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(&img.pixels()[0..4], &[255, 0, 0, 255]);
        assert_eq!(&img.pixels()[4..8], &[0, 255, 0, 255]);
        assert_eq!(&img.pixels()[8..12], &[0, 0, 255, 255]);
        assert_eq!(&img.pixels()[12..16], &[255, 255, 255, 255]);
    }
}
