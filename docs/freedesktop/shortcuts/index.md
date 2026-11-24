[Jump to content](#_content)[Shortcuts Specification](#id-1)[Show Contents: Shortcuts Specification](#id-1)[Top](#)
1. [1 About](#about)
2. [2 Specification](#specification)

## Shortcuts Specification [#](#id-1)

- File Name: shortcuts-spec.xml
- ID: no ID found

Author: Aleix Pol Gonzalez `<aleixpol@kde.org>`
Publication Date: 2022-05-11, Version: 0.1 (DRAFT)

- **[1 About](#about)**
- **[2 Specification](#specification)**

## 1 About [#](#about)

- File Name: shortcuts-spec.xml
- ID: about

 The purpose of this specification is to specify how shortcuts can be specified between components. Be it for global shortcuts in the XDG portals or to specify how we need an application to get started, we need shared means to communicate these triggers.

 This document contemplates simple keyboard shortcuts right now. Extending this specification with more complex cases or input beyond the traditional keyboard devices is something that can be included in future versions of the spec.

 The identifiers used in this document all come from the xkbcommon project, used by most implementations of keyboard input. You can find more information here: https://xkbcommon.org

## 2 Specification [#](#specification)

- File Name: shortcuts-spec.xml
- ID: specification

 A shortcut is comprised by a set of modifiers (namely CTRL, ALT, SHIFT, NUM and LOGO, as defined as XKB_MOD_NAME_* in [xkbcommon-names.h (https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-names.h)](https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-names.h)) together with a key identifier, joined by a + sign.

 Identifiers are taken from [xkbcommon-keysyms.h (https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-keysyms.h)](https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-keysyms.h) without the XKB_KEY_* prefix. Identifiers only contain alphanumerical values and underscore (_).

 Examples: * a: Signifies the `XKB_KEY_a` key on a keyboard * CTRL+a: `Control` modifier is pressed, then the `XKB_KEY_a` key * CTRL+SHIFT+a: `Control` and `Shift` modifiers are pressed, then the `XKB_KEY_a` key. * CTRL+ALT+Return: `Control` and `Alt` modifiers are pressed, then the `XKB_KEY_Return` key.

 Here we are defining the language to communicate a desired shortcut between components in a standard way. It’s up to the implementation to produce the expected behaviour to their users. The implementation will be responsible for the effort of translating the entered shortcut into something that can be expressed in the user’s keyboard layout.

 Limit shortcuts to the base (or depressed, in terms of xkbcommon’s documentation) layer, so it’s easy to understand what it refers to. What a base layer is will depend on the specific layout in use, it’s up to the implementation to decide how to resolve these.

 Shortcuts will be parsed until an undocumented character is found.

© 2026 Freedesktop.org
