//! Freedesktop D-Bus session service for SpectreOS Keyboard (`org.spectreos.OSK0`).
//!
//! Implements the Freedesktop D-Bus Specification (`docs/freedesktop/dbus/dbus-specification.md`)
//! using `libsystemd`'s `sd-bus` C ABI (matching Wisp's zero-bloat notification daemon):
//! - Primary service: bus name `org.spectreos.OSK0`, object path `/org/spectreos/OSK0`,
//!   interface `org.spectreos.OSK0`:
//!   - Method `SetVisible(in b visible)`
//!   - Method `SetLayout(in s layout)`
//!   - Property `Visible` (`b`, readwrite, `EmitsChangedSignal = true`)
//!   - Property `ActiveLayout` (`s`, read, `EmitsChangedSignal = true`)
//! - Standard interfaces:
//!   - `org.freedesktop.DBus.Peer` (`Ping`, `GetMachineId`, handled by `sd-bus`)
//!   - `org.freedesktop.DBus.Introspectable` (`Introspect`)
//!   - `org.freedesktop.DBus.Properties` (`Get`, `Set`, `GetAll`, `PropertiesChanged`)
//! - Optional compatibility alias (enabled unless `--no-compat-dbus`):
//!   - Best-effort claims `sm.puri.OSK0` and exports `/sm/puri/OSK0` (`sm.puri.OSK0`) so
//!     third-party Linux mobile apps that use the legacy Phosh/Squeekboard D-Bus path
//!     interoperate seamlessly without requiring Phosh or Stevia.

use std::{
    collections::VecDeque,
    ffi::{CStr, CString, c_char, c_int, c_void},
    os::fd::RawFd,
    ptr,
};

pub const PRIMARY_BUS_NAME: &str = "org.spectreos.OSK0";
pub const PRIMARY_OBJECT_PATH: &str = "/org/spectreos/OSK0";
pub const PRIMARY_INTERFACE: &str = "org.spectreos.OSK0";

pub const COMPAT_BUS_NAME: &str = "sm.puri.OSK0";
pub const COMPAT_OBJECT_PATH: &str = "/sm/puri/OSK0";
pub const COMPAT_INTERFACE: &str = "sm.puri.OSK0";

const SD_BUS_NAME_ALLOW_REPLACEMENT: u64 = 1 << 0;
const SD_BUS_NAME_REPLACE_EXISTING: u64 = 1 << 1;

#[repr(C)]
struct SdBus {
    _private: [u8; 0],
}

#[repr(C)]
struct SdBusSlot {
    _private: [u8; 0],
}

#[repr(C)]
struct SdBusMessage {
    _private: [u8; 0],
}

#[repr(C)]
struct SdBusError {
    name: *const c_char,
    message: *const c_char,
    _need_free: c_int,
}

type SdBusMessageHandler =
    unsafe extern "C" fn(*mut SdBusMessage, *mut c_void, *mut SdBusError) -> c_int;

#[link(name = "systemd")]
unsafe extern "C" {
    fn sd_bus_default_user(ret: *mut *mut SdBus) -> c_int;
    fn sd_bus_request_name(bus: *mut SdBus, name: *const c_char, flags: u64) -> c_int;
    fn sd_bus_add_object(
        bus: *mut SdBus,
        slot: *mut *mut SdBusSlot,
        path: *const c_char,
        callback: SdBusMessageHandler,
        userdata: *mut c_void,
    ) -> c_int;
    fn sd_bus_slot_unref(slot: *mut SdBusSlot) -> *mut SdBusSlot;
    fn sd_bus_get_fd(bus: *mut SdBus) -> c_int;
    fn sd_bus_process(bus: *mut SdBus, ret: *mut *mut SdBusMessage) -> c_int;
    fn sd_bus_flush(bus: *mut SdBus) -> c_int;
    fn sd_bus_flush_close_unref(bus: *mut SdBus) -> *mut SdBus;
    fn sd_bus_send(bus: *mut SdBus, m: *mut SdBusMessage, cookie: *mut u64) -> c_int;

    fn sd_bus_message_is_method_call(
        m: *mut SdBusMessage,
        interface: *const c_char,
        member: *const c_char,
    ) -> c_int;
    fn sd_bus_message_get_path(m: *mut SdBusMessage) -> *const c_char;
    fn sd_bus_message_read_basic(m: *mut SdBusMessage, ty: c_char, p: *mut c_void) -> c_int;
    fn sd_bus_message_enter_container(
        m: *mut SdBusMessage,
        ty: c_char,
        contents: *const c_char,
    ) -> c_int;
    fn sd_bus_message_exit_container(m: *mut SdBusMessage) -> c_int;
    fn sd_bus_message_peek_type(
        m: *mut SdBusMessage,
        ty: *mut c_char,
        contents: *mut *const c_char,
    ) -> c_int;

    fn sd_bus_message_new_method_return(
        call: *mut SdBusMessage,
        m: *mut *mut SdBusMessage,
    ) -> c_int;
    fn sd_bus_message_new_method_error(
        call: *mut SdBusMessage,
        m: *mut *mut SdBusMessage,
        e: *const SdBusError,
    ) -> c_int;
    fn sd_bus_message_new_signal(
        bus: *mut SdBus,
        m: *mut *mut SdBusMessage,
        path: *const c_char,
        interface: *const c_char,
        member: *const c_char,
    ) -> c_int;
    fn sd_bus_message_append_basic(m: *mut SdBusMessage, ty: c_char, p: *const c_void) -> c_int;
    fn sd_bus_message_open_container(
        m: *mut SdBusMessage,
        ty: c_char,
        contents: *const c_char,
    ) -> c_int;
    fn sd_bus_message_close_container(m: *mut SdBusMessage) -> c_int;
    fn sd_bus_message_unref(m: *mut SdBusMessage) -> *mut SdBusMessage;
}

pub const PRIMARY_INTROSPECT_XML: &str = r#"<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node name="/org/spectreos/OSK0">
  <interface name="org.freedesktop.DBus.Peer">
    <method name="Ping"/>
    <method name="GetMachineId">
      <arg name="machine_uuid" type="s" direction="out"/>
    </method>
  </interface>
  <interface name="org.freedesktop.DBus.Introspectable">
    <method name="Introspect">
      <arg name="xml_data" type="s" direction="out"/>
    </method>
  </interface>
  <interface name="org.freedesktop.DBus.Properties">
    <method name="Get">
      <arg name="interface_name" type="s" direction="in"/>
      <arg name="property_name" type="s" direction="in"/>
      <arg name="value" type="v" direction="out"/>
    </method>
    <method name="Set">
      <arg name="interface_name" type="s" direction="in"/>
      <arg name="property_name" type="s" direction="in"/>
      <arg name="value" type="v" direction="in"/>
    </method>
    <method name="GetAll">
      <arg name="interface_name" type="s" direction="in"/>
      <arg name="props" type="a{sv}" direction="out"/>
    </method>
    <signal name="PropertiesChanged">
      <arg name="interface_name" type="s"/>
      <arg name="changed_properties" type="a{sv}"/>
      <arg name="invalidated_properties" type="as"/>
    </signal>
  </interface>
  <interface name="org.spectreos.OSK0">
    <method name="SetVisible">
      <arg name="visible" type="b" direction="in"/>
    </method>
    <method name="SetLayout">
      <arg name="layout" type="s" direction="in"/>
    </method>
    <property name="Visible" type="b" access="readwrite"/>
    <property name="ActiveLayout" type="s" access="read"/>
  </interface>
</node>
"#;

pub const COMPAT_INTROSPECT_XML: &str = r#"<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node name="/sm/puri/OSK0">
  <interface name="org.freedesktop.DBus.Introspectable">
    <method name="Introspect">
      <arg name="xml_data" type="s" direction="out"/>
    </method>
  </interface>
  <interface name="org.freedesktop.DBus.Properties">
    <method name="Get">
      <arg name="interface_name" type="s" direction="in"/>
      <arg name="property_name" type="s" direction="in"/>
      <arg name="value" type="v" direction="out"/>
    </method>
    <method name="Set">
      <arg name="interface_name" type="s" direction="in"/>
      <arg name="property_name" type="s" direction="in"/>
      <arg name="value" type="v" direction="in"/>
    </method>
    <method name="GetAll">
      <arg name="interface_name" type="s" direction="in"/>
      <arg name="props" type="a{sv}" direction="out"/>
    </method>
    <signal name="PropertiesChanged">
      <arg name="interface_name" type="s"/>
      <arg name="changed_properties" type="a{sv}"/>
      <arg name="invalidated_properties" type="as"/>
    </signal>
  </interface>
  <interface name="sm.puri.OSK0">
    <method name="SetVisible">
      <arg name="visible" type="b" direction="in"/>
    </method>
    <property name="Visible" type="b" access="readwrite"/>
  </interface>
</node>
"#;

/// Inbound command received over D-Bus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OskDbusEvent {
    SetVisible(bool),
    SetLayout(String),
}

struct SharedState {
    bus: *mut SdBus,
    visible: bool,
    active_layout: String,
    events: VecDeque<OskDbusEvent>,
}

unsafe fn read_bool(m: *mut SdBusMessage) -> Option<bool> {
    let mut v: c_int = 0;
    let r = unsafe { sd_bus_message_read_basic(m, b'b' as c_char, (&mut v as *mut c_int).cast()) };
    (r > 0).then_some(v != 0)
}

unsafe fn read_string(m: *mut SdBusMessage) -> Option<String> {
    let mut p: *const c_char = ptr::null();
    let r = unsafe {
        sd_bus_message_read_basic(m, b's' as c_char, (&mut p as *mut *const c_char).cast())
    };
    if r <= 0 {
        return None;
    }
    if p.is_null() {
        return Some(String::new());
    }
    Some(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
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

unsafe fn append_variant_bool(m: *mut SdBusMessage, value: bool) {
    let v: c_int = if value { 1 } else { 0 };
    unsafe {
        if sd_bus_message_open_container(m, b'v' as c_char, c"b".as_ptr()) >= 0 {
            sd_bus_message_append_basic(m, b'b' as c_char, (&v as *const c_int).cast());
            sd_bus_message_close_container(m);
        }
    }
}

unsafe fn append_variant_string(m: *mut SdBusMessage, value: &str) {
    let Ok(cval) = CString::new(value) else {
        return;
    };
    unsafe {
        if sd_bus_message_open_container(m, b'v' as c_char, c"s".as_ptr()) >= 0 {
            sd_bus_message_append_basic(m, b's' as c_char, cval.as_ptr().cast());
            sd_bus_message_close_container(m);
        }
    }
}

unsafe fn emit_property_changed_bool(
    bus: *mut SdBus,
    path: &CStr,
    iface: &CStr,
    prop: &CStr,
    value: bool,
) {
    let mut sig: *mut SdBusMessage = ptr::null_mut();
    if unsafe {
        sd_bus_message_new_signal(
            bus,
            &mut sig,
            path.as_ptr(),
            c"org.freedesktop.DBus.Properties".as_ptr(),
            c"PropertiesChanged".as_ptr(),
        )
    } >= 0
        && !sig.is_null()
    {
        unsafe {
            sd_bus_message_append_basic(sig, b's' as c_char, iface.as_ptr().cast());
            if sd_bus_message_open_container(sig, b'a' as c_char, c"{sv}".as_ptr()) >= 0 {
                if sd_bus_message_open_container(sig, b'e' as c_char, c"sv".as_ptr()) >= 0 {
                    sd_bus_message_append_basic(sig, b's' as c_char, prop.as_ptr().cast());
                    append_variant_bool(sig, value);
                    sd_bus_message_close_container(sig);
                }
                sd_bus_message_close_container(sig);
            }
            if sd_bus_message_open_container(sig, b'a' as c_char, c"s".as_ptr()) >= 0 {
                sd_bus_message_close_container(sig);
            }
            sd_bus_send(bus, sig, ptr::null_mut());
            sd_bus_message_unref(sig);
        }
    }
}

unsafe fn emit_property_changed_string(
    bus: *mut SdBus,
    path: &CStr,
    iface: &CStr,
    prop: &CStr,
    value: &str,
) {
    let mut sig: *mut SdBusMessage = ptr::null_mut();
    if unsafe {
        sd_bus_message_new_signal(
            bus,
            &mut sig,
            path.as_ptr(),
            c"org.freedesktop.DBus.Properties".as_ptr(),
            c"PropertiesChanged".as_ptr(),
        )
    } >= 0
        && !sig.is_null()
    {
        unsafe {
            sd_bus_message_append_basic(sig, b's' as c_char, iface.as_ptr().cast());
            if sd_bus_message_open_container(sig, b'a' as c_char, c"{sv}".as_ptr()) >= 0 {
                if sd_bus_message_open_container(sig, b'e' as c_char, c"sv".as_ptr()) >= 0 {
                    sd_bus_message_append_basic(sig, b's' as c_char, prop.as_ptr().cast());
                    append_variant_string(sig, value);
                    sd_bus_message_close_container(sig);
                }
                sd_bus_message_close_container(sig);
            }
            if sd_bus_message_open_container(sig, b'a' as c_char, c"s".as_ptr()) >= 0 {
                sd_bus_message_close_container(sig);
            }
            sd_bus_send(bus, sig, ptr::null_mut());
            sd_bus_message_unref(sig);
        }
    }
}

unsafe extern "C" fn handle_osk_message(
    m: *mut SdBusMessage,
    userdata: *mut c_void,
    _ret_error: *mut SdBusError,
) -> c_int {
    if m.is_null() || userdata.is_null() {
        return 0;
    }
    let state = unsafe { &mut *userdata.cast::<SharedState>() };
    let bus = state.bus;
    let path_ptr = unsafe { sd_bus_message_get_path(m) };
    let is_compat_path = !path_ptr.is_null()
        && unsafe { CStr::from_ptr(path_ptr) }.to_str() == Ok(COMPAT_OBJECT_PATH);

    // 1. org.freedesktop.DBus.Introspectable.Introspect
    if unsafe {
        sd_bus_message_is_method_call(
            m,
            c"org.freedesktop.DBus.Introspectable".as_ptr(),
            c"Introspect".as_ptr(),
        )
    } > 0
    {
        let xml = if is_compat_path {
            COMPAT_INTROSPECT_XML
        } else {
            PRIMARY_INTROSPECT_XML
        };
        unsafe { send_string_return(bus, m, xml) };
        return 1;
    }

    // 2. SetVisible(b) on primary (`org.spectreos.OSK0`) or compat (`sm.puri.OSK0`) interface
    let is_set_visible = unsafe {
        sd_bus_message_is_method_call(m, c"org.spectreos.OSK0".as_ptr(), c"SetVisible".as_ptr()) > 0
            || sd_bus_message_is_method_call(m, c"sm.puri.OSK0".as_ptr(), c"SetVisible".as_ptr())
                > 0
    };
    if is_set_visible {
        let Some(vis) = (unsafe { read_bool(m) }) else {
            unsafe {
                send_error(
                    bus,
                    m,
                    c"org.freedesktop.DBus.Error.InvalidArgs",
                    c"Expected boolean visible argument",
                );
            }
            return 1;
        };
        state.events.push_back(OskDbusEvent::SetVisible(vis));
        unsafe { send_empty_return(bus, m) };
        return 1;
    }

    // 3. SetLayout(s) on `org.spectreos.OSK0`
    if unsafe {
        sd_bus_message_is_method_call(m, c"org.spectreos.OSK0".as_ptr(), c"SetLayout".as_ptr())
    } > 0
    {
        let Some(layout) = (unsafe { read_string(m) }) else {
            unsafe {
                send_error(
                    bus,
                    m,
                    c"org.freedesktop.DBus.Error.InvalidArgs",
                    c"Expected string layout argument",
                );
            }
            return 1;
        };
        state.events.push_back(OskDbusEvent::SetLayout(layout));
        unsafe { send_empty_return(bus, m) };
        return 1;
    }

    // 4. org.freedesktop.DBus.Properties.Get(s interface, s property) -> v
    if unsafe {
        sd_bus_message_is_method_call(
            m,
            c"org.freedesktop.DBus.Properties".as_ptr(),
            c"Get".as_ptr(),
        )
    } > 0
    {
        let iface = unsafe { read_string(m) }.unwrap_or_default();
        let prop = unsafe { read_string(m) }.unwrap_or_default();
        let valid_iface = iface.is_empty()
            || iface == PRIMARY_INTERFACE
            || iface == COMPAT_INTERFACE;
        if !valid_iface {
            unsafe {
                send_error(
                    bus,
                    m,
                    c"org.freedesktop.DBus.Error.UnknownInterface",
                    c"Unknown interface",
                );
            }
            return 1;
        }
        let mut reply: *mut SdBusMessage = ptr::null_mut();
        match prop.as_str() {
            "Visible" => {
                if unsafe { sd_bus_message_new_method_return(m, &mut reply) } >= 0
                    && !reply.is_null()
                {
                    unsafe {
                        append_variant_bool(reply, state.visible);
                        sd_bus_send(bus, reply, ptr::null_mut());
                        sd_bus_message_unref(reply);
                    }
                }
            }
            "ActiveLayout" if !is_compat_path => {
                if unsafe { sd_bus_message_new_method_return(m, &mut reply) } >= 0
                    && !reply.is_null()
                {
                    unsafe {
                        append_variant_string(reply, &state.active_layout);
                        sd_bus_send(bus, reply, ptr::null_mut());
                        sd_bus_message_unref(reply);
                    }
                }
            }
            _ => unsafe {
                send_error(
                    bus,
                    m,
                    c"org.freedesktop.DBus.Error.UnknownProperty",
                    c"Unknown property",
                );
            },
        }
        return 1;
    }

    // 5. org.freedesktop.DBus.Properties.Set(s interface, s property, v value)
    if unsafe {
        sd_bus_message_is_method_call(
            m,
            c"org.freedesktop.DBus.Properties".as_ptr(),
            c"Set".as_ptr(),
        )
    } > 0
    {
        let iface = unsafe { read_string(m) }.unwrap_or_default();
        let prop = unsafe { read_string(m) }.unwrap_or_default();
        if !(iface.is_empty() || iface == PRIMARY_INTERFACE || iface == COMPAT_INTERFACE) {
            unsafe {
                send_error(
                    bus,
                    m,
                    c"org.freedesktop.DBus.Error.UnknownInterface",
                    c"Unknown interface",
                );
            }
            return 1;
        }
        if prop == "Visible" {
            let mut vtype: c_char = 0;
            let mut contents: *const c_char = ptr::null();
            if unsafe { sd_bus_message_peek_type(m, &mut vtype, &mut contents) } > 0
                && vtype == b'v' as c_char
                && !contents.is_null()
                && unsafe { CStr::from_ptr(contents) }.to_str() == Ok("b")
                && unsafe { sd_bus_message_enter_container(m, b'v' as c_char, contents) } > 0
            {
                let vis = unsafe { read_bool(m) };
                unsafe { sd_bus_message_exit_container(m) };
                if let Some(vis) = vis {
                    state.events.push_back(OskDbusEvent::SetVisible(vis));
                    unsafe { send_empty_return(bus, m) };
                    return 1;
                }
            }
            unsafe {
                send_error(
                    bus,
                    m,
                    c"org.freedesktop.DBus.Error.InvalidArgs",
                    c"Visible property requires boolean variant",
                );
            }
            return 1;
        }
        unsafe {
            send_error(
                bus,
                m,
                c"org.freedesktop.DBus.Error.PropertyReadOnly",
                c"Property is not writable",
            );
        }
        return 1;
    }

    // 6. org.freedesktop.DBus.Properties.GetAll(s interface) -> a{sv}
    if unsafe {
        sd_bus_message_is_method_call(
            m,
            c"org.freedesktop.DBus.Properties".as_ptr(),
            c"GetAll".as_ptr(),
        )
    } > 0
    {
        let iface = unsafe { read_string(m) }.unwrap_or_default();
        let mut reply: *mut SdBusMessage = ptr::null_mut();
        if unsafe { sd_bus_message_new_method_return(m, &mut reply) } >= 0 && !reply.is_null() {
            unsafe {
                if sd_bus_message_open_container(reply, b'a' as c_char, c"{sv}".as_ptr()) >= 0 {
                    let include_primary =
                        iface.is_empty() || iface == PRIMARY_INTERFACE || iface == COMPAT_INTERFACE;
                    if include_primary {
                        if sd_bus_message_open_container(reply, b'e' as c_char, c"sv".as_ptr()) >= 0
                        {
                            sd_bus_message_append_basic(
                                reply,
                                b's' as c_char,
                                c"Visible".as_ptr().cast(),
                            );
                            append_variant_bool(reply, state.visible);
                            sd_bus_message_close_container(reply);
                        }
                        if !is_compat_path && iface != COMPAT_INTERFACE {
                            if sd_bus_message_open_container(reply, b'e' as c_char, c"sv".as_ptr())
                                >= 0
                            {
                                sd_bus_message_append_basic(
                                    reply,
                                    b's' as c_char,
                                    c"ActiveLayout".as_ptr().cast(),
                                );
                                append_variant_string(reply, &state.active_layout);
                                sd_bus_message_close_container(reply);
                            }
                        }
                    }
                    sd_bus_message_close_container(reply);
                }
                sd_bus_send(bus, reply, ptr::null_mut());
                sd_bus_message_unref(reply);
            }
        }
        return 1;
    }

    0
}

/// Live session D-Bus service owned by `spectre-keyboard`.
pub struct OskDbusServer {
    bus: *mut SdBus,
    primary_slot: *mut SdBusSlot,
    compat_slot: *mut SdBusSlot,
    compat_active: bool,
    shared: Box<SharedState>,
    fd: RawFd,
}

impl OskDbusServer {
    pub fn start(initial_visible: bool, initial_layout: &str, compat_alias: bool) -> Result<Self, String> {
        let mut bus: *mut SdBus = ptr::null_mut();
        let r = unsafe { sd_bus_default_user(&mut bus) };
        if r < 0 || bus.is_null() {
            return Err(format!("sd_bus_default_user failed: errno={}", -r));
        }

        let mut shared = Box::new(SharedState {
            bus,
            visible: initial_visible,
            active_layout: initial_layout.to_string(),
            events: VecDeque::new(),
        });

        let mut primary_slot: *mut SdBusSlot = ptr::null_mut();
        let r_obj = unsafe {
            sd_bus_add_object(
                bus,
                &mut primary_slot,
                c"/org/spectreos/OSK0".as_ptr(),
                handle_osk_message,
                (&mut *shared as *mut SharedState).cast(),
            )
        };
        if r_obj < 0 {
            unsafe { sd_bus_flush_close_unref(bus) };
            return Err(format!("sd_bus_add_object(/org/spectreos/OSK0) failed: errno={}", -r_obj));
        }

        let flags = SD_BUS_NAME_REPLACE_EXISTING | SD_BUS_NAME_ALLOW_REPLACEMENT;
        let r_name = unsafe { sd_bus_request_name(bus, c"org.spectreos.OSK0".as_ptr(), flags) };
        if r_name < 0 {
            unsafe {
                sd_bus_slot_unref(primary_slot);
                sd_bus_flush_close_unref(bus);
            }
            return Err(format!(
                "sd_bus_request_name(org.spectreos.OSK0) failed: errno={}",
                -r_name
            ));
        }

        let mut compat_slot: *mut SdBusSlot = ptr::null_mut();
        let mut compat_active = false;
        if compat_alias {
            let rc_obj = unsafe {
                sd_bus_add_object(
                    bus,
                    &mut compat_slot,
                    c"/sm/puri/OSK0".as_ptr(),
                    handle_osk_message,
                    (&mut *shared as *mut SharedState).cast(),
                )
            };
            if rc_obj >= 0 {
                let rc_name = unsafe { sd_bus_request_name(bus, c"sm.puri.OSK0".as_ptr(), flags) };
                if rc_name >= 0 {
                    compat_active = true;
                } else {
                    unsafe { sd_bus_slot_unref(compat_slot) };
                    compat_slot = ptr::null_mut();
                }
            }
        }

        let fd = unsafe { sd_bus_get_fd(bus) };
        if fd < 0 {
            unsafe {
                if !compat_slot.is_null() {
                    sd_bus_slot_unref(compat_slot);
                }
                sd_bus_slot_unref(primary_slot);
                sd_bus_flush_close_unref(bus);
            }
            return Err(format!("sd_bus_get_fd failed: errno={}", -fd));
        }

        eprintln!(
            "spectre-keyboard: claimed D-Bus {PRIMARY_BUS_NAME} ({PRIMARY_OBJECT_PATH}, compat_sm_puri={compat_active}, fd={fd})"
        );

        Ok(Self {
            bus,
            primary_slot,
            compat_slot,
            compat_active,
            shared,
            fd,
        })
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn process(&mut self) -> Vec<OskDbusEvent> {
        loop {
            let r = unsafe { sd_bus_process(self.bus, ptr::null_mut()) };
            if r <= 0 {
                break;
            }
        }
        unsafe { sd_bus_flush(self.bus) };
        self.shared.events.drain(..).collect()
    }

    pub fn set_visible_state(&mut self, visible: bool) {
        if self.shared.visible == visible {
            return;
        }
        self.shared.visible = visible;
        unsafe {
            emit_property_changed_bool(
                self.bus,
                c"/org/spectreos/OSK0",
                c"org.spectreos.OSK0",
                c"Visible",
                visible,
            );
            if self.compat_active {
                emit_property_changed_bool(
                    self.bus,
                    c"/sm/puri/OSK0",
                    c"sm.puri.OSK0",
                    c"Visible",
                    visible,
                );
            }
            sd_bus_flush(self.bus);
        }
    }

    pub fn set_active_layout_state(&mut self, layout: &str) {
        if self.shared.active_layout == layout {
            return;
        }
        self.shared.active_layout = layout.to_string();
        unsafe {
            emit_property_changed_string(
                self.bus,
                c"/org/spectreos/OSK0",
                c"org.spectreos.OSK0",
                c"ActiveLayout",
                layout,
            );
            sd_bus_flush(self.bus);
        }
    }
}

impl Drop for OskDbusServer {
    fn drop(&mut self) {
        unsafe {
            if !self.compat_slot.is_null() {
                sd_bus_slot_unref(self.compat_slot);
            }
            if !self.primary_slot.is_null() {
                sd_bus_slot_unref(self.primary_slot);
            }
            if !self.bus.is_null() {
                sd_bus_flush_close_unref(self.bus);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn introspection_xml_declares_spec_standard_and_osk_interfaces() {
        assert!(PRIMARY_INTROSPECT_XML.contains(r#"interface name="org.spectreos.OSK0""#));
        assert!(PRIMARY_INTROSPECT_XML.contains(r#"method name="SetVisible""#));
        assert!(PRIMARY_INTROSPECT_XML.contains(r#"method name="SetLayout""#));
        assert!(PRIMARY_INTROSPECT_XML.contains(r#"property name="Visible""#));
        assert!(PRIMARY_INTROSPECT_XML.contains(r#"interface name="org.freedesktop.DBus.Properties""#));
        assert!(PRIMARY_INTROSPECT_XML.contains(r#"interface name="org.freedesktop.DBus.Introspectable""#));
        assert!(COMPAT_INTROSPECT_XML.contains(r#"interface name="sm.puri.OSK0""#));
    }

    #[test]
    #[ignore = "requires an active DBUS_SESSION_BUS_ADDRESS (run under dbus-run-session)"]
    fn live_session_bus_handles_set_visible_layout_and_properties() {
        use std::process::Command;
        use std::time::{Duration, Instant};

        let mut srv = OskDbusServer::start(false, "us", true).expect("start OskDbusServer");
        let client_bin = std::env::var("SPECTRE_TEST_SHIRAZ_DBUS_CLIENT").ok();

        let child_handle = std::thread::spawn(move || {
            let introspect = Command::new("gdbus")
                .args([
                    "introspect",
                    "--session",
                    "--dest",
                    PRIMARY_BUS_NAME,
                    "--object-path",
                    PRIMARY_OBJECT_PATH,
                ])
                .output()
                .expect("gdbus introspect");
            assert!(introspect.status.success(), "introspect failed: {introspect:?}");
            let xml_out = String::from_utf8_lossy(&introspect.stdout);
            assert!(xml_out.contains("SetVisible"), "missing SetVisible in {xml_out}");

            let set_layout = Command::new("gdbus")
                .args([
                    "call",
                    "--session",
                    "--dest",
                    PRIMARY_BUS_NAME,
                    "--object-path",
                    PRIMARY_OBJECT_PATH,
                    "--method",
                    "org.spectreos.OSK0.SetLayout",
                    "ir",
                ])
                .output()
                .expect("gdbus SetLayout");
            assert!(set_layout.status.success(), "SetLayout failed: {set_layout:?}");

            if let Some(bin) = client_bin {
                let st = Command::new(bin).status().expect("run Shiraz C++ D-Bus OSK client");
                assert!(st.success(), "Shiraz C++ D-Bus OSK client failed");
            } else {
                for vis in ["true", "false"] {
                    let out = Command::new("gdbus")
                        .args([
                            "call",
                            "--session",
                            "--dest",
                            PRIMARY_BUS_NAME,
                            "--object-path",
                            PRIMARY_OBJECT_PATH,
                            "--method",
                            "org.spectreos.OSK0.SetVisible",
                            vis,
                        ])
                        .output()
                        .expect("gdbus SetVisible");
                    assert!(out.status.success());
                }
            }
        });

        let mut received = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && received.len() < 3 {
            let evs = srv.process();
            for ev in evs {
                match &ev {
                    OskDbusEvent::SetVisible(v) => srv.set_visible_state(*v),
                    OskDbusEvent::SetLayout(l) => srv.set_active_layout_state(l),
                }
                received.push(ev);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        child_handle.join().expect("client thread");
        assert_eq!(
            received,
            vec![
                OskDbusEvent::SetLayout("ir".into()),
                OskDbusEvent::SetVisible(true),
                OskDbusEvent::SetVisible(false),
            ]
        );
    }
}
