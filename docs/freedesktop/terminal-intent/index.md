[Jump to content](#_content)[Terminal Intent](#id-1)[Show Contents: Terminal Intent](#id-1)[Top](#)
1. [1 About](#about)
2. [2 Specification](#specification)
3. [3 Example Usage](#example-usage)

## Terminal Intent [#](#id-1)

- File Name: terminal-intent-spec.xml
- ID: no ID found

Authors: Zander Brown `<zbrown@gnome.org>`David Faure `<faure@kde.org>`Thayne McCombs `<>`Sebastian Wick `<swick@gnome.org>`
Publication Date: 2026-10-02, Version: 0.1 (DRAFT)

- **[1 About](#about)**
- **[2 Specification](#specification)**
- **[3 Example Usage](#example-usage)**

## 1 About [#](#about)

- File Name: terminal-intent-spec.xml
- ID: about

 Non-graphical applications (‘terminal apps’) must be launched in a terminal emulator. This specification aims to provide a common interface for launching terminal emulators and optionally executing commands in them.

 The [ Intent App specification (https://specifications.freedesktop.org/intent-apps/latest/)](https://specifications.freedesktop.org/intent-apps/latest/) provides the means to discover and select apps which implement this interface. For general documentation on Intents, see the [ Desktop Entry specifications (https://specifications.freedesktop.org/desktop-entry/latest/interfaces.html)](https://specifications.freedesktop.org/desktop-entry/latest/interfaces.html).

## 2 Specification [#](#specification)

- File Name: terminal-intent-spec.xml
- ID: specification

 Applications implementing the Terminal Intent must indicate support for it by adding the string `org.freedesktop.Terminal1` to the `Implements` key in their Desktop Entry File.

 The app must be `DBusActivatable`, and the app's desktop file must be named using the D-Bus "reverse DNS" convention. See [ D-Bus Activation  (https://specifications.freedesktop.org/desktop-entry/latest/dbus.html)](https://specifications.freedesktop.org/desktop-entry/latest/dbus.html).

 The app must export the D-Bus interface `org.freedesktop.Terminal1` on the same object path as it exports the `org.freedesktop.Application` interface.

```

<!DOCTYPE node PUBLIC
      "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
      "https://dbus.freedesktop.org/doc/introspect.dtd" >
<node>
  <interface name="org.freedesktop.Terminal1">
    <method name="LaunchCommand">
      <arg type='aa{sv}' name='commands' direction='in' />
      <arg type='ay' name='desktop_entry' direction='in' />
      <arg type='a{sv}' name='options' direction='in' />
      <arg type='a{sv}' name='platform_data' direction='in' />
    </method>
  </interface>
</node>

```

 Launch a terminal with a given command.

 As paths may contain non-UTF-8 data, parameters use nul-terminated `ay` bytestrings rather than `s`. Implementations should take care when handling these bytestrings.

 Returns an empty reply on success, and a relevant D-Bus error on failure, as specified in each of the parameters’ descriptions.

###### Table 1: LaunchCommand arguments [#](#id-1.3.9)

| Key | Type | Description |
| --- | --- | --- |
| `exec` | `aay` | Argument vector (`execve(2)`-style), or zero length. If the vector has zero length, or the key is not provided, a regular shell session should be launched instead. |
| `env` | `aay` | Environment list (`execve(2)`-style), or zero length. These `key=value` values should modify (rather than substitute) the environment an implementation would otherwise use when invoking `exec`. If the vector has zero length, or the key is not provided, the environment should not be modified. If any of the values do not match the `key=value` format, an org.freedesktop.DBus.Error.InvalidArgs error is returned. |
| `working_directory` | `ay` | Specifies the directory `exec` or regular shell session should be launched in. Relative paths should be considered in relation to `$HOME` or equivalent. If the bytestring has zero length, or the key is not provided, implementations should use `$HOME` or equivalent instead. When launching a Desktop Entry, the value of `Path` should be used (if present and `working_directory` has zero length or is not provided). Otherwise the `working_directory` value should be used, and if both are empty, fallback to `$HOME` or equivalent. |

`desktop_entry``ay` The full path to the Desktop Entry of the terminal application being launched. Zero length when opening an empty shell session or running commands not associated with any Desktop Entry. Implementations may use this for things such as window title, by accessing the `Name` key, though such behaviour is optional. If the bytestring has non-zero length and is not a valid path to a Desktop Entry, an org.freedesktop.DBus.Error.InvalidArgs error is returned. `options``a{sv}` Additional options. Implementations should try to implement as many of these options as possible. To allow the addition of new options in the future, implementations should ignore unknown options.

###### Table 3: Supported options [#](#id-1.3.9.2.2.3.3.1)

| Option | Type | Description |
| --- | --- | --- |
| `keep-terminal-open` | `b` | Terminal should be held open after `exec` exits so a user can inspect the output. If the option is not provided, or is false, the terminal should not be kept open. |

`platform_data``a{sv}` Meta information to help with platform integration, such as startup notification. Keys are specified for [ D-Bus activation in the Desktop Entry Specification (https://specifications.freedesktop.org/desktop-entry/latest/dbus.html)](https://specifications.freedesktop.org/desktop-entry/latest/dbus.html). Currently supported keys are: `desktop-startup-id` and `activation-token`. To allow the addition of new keys in the future, implementations should ignore unknown keys.

## 3 Example Usage [#](#example-usage)

- File Name: terminal-intent-spec.xml
- ID: example-usage

 Given a hypothetical text-based editor

```

[Desktop Entry]
Version=1.0
Type=Application
Terminal=true
Name=Editor
Comment=A terminal text editor
Exec=xdg-editor %f
Path=/opt/xdg/editor
Icon=org.freedesktop.DuperEditor
MimeType=text/plain

```

 And a chosen terminal emulator

```

[Desktop Entry]
Version=1.0
Type=Application
Name=SuperTerm
Comment=The best terminal available!
Exec=xdg-super-term
Icon=org.freedesktop.SuperTerm
Implements=org.freedesktop.Terminal1

```

 A file `sweden.txt` would be opened in Editor (via SuperTerm) with a call resembling

```

Name: org.freedesktop.SuperTerm
Path: /org/freedesktop/SuperTerm
org.freedesktop.Terminal1.LaunchCommand([
                                          {
                                            'exec': [b'xdg-editor', b'/path/to/sweden.txt'],
                                            'env': [],
                                            'working_directory': b'/opt/xdg/editor'
                                          }
                                        ],
                                        b'/usr/share/applications/org.freedesktop.Editor.desktop',
                                        {},
                                        {'desktop-startup-id': 'token'})

```

© 2026 Freedesktop.org
