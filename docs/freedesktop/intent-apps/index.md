[Jump to content](#_content)[Application Intents Specification](#id-1)[Show Contents: Application Intents Specification](#id-1)[Top](#)
1. [1 Introduction](#introduction)
2. [2 File name and location](#file)
3. [3 Default Application](#default)
4. [4 Scopes](#scopes)
5. [5 Cache](#cache)

## Application Intents Specification [#](#id-1)

Authors: David Faure `<faure@kde.org>`Sebastian Wick `<swick@gnome.org>`
Publication Date: 2025-09-23, Version: 0.1 (DRAFT)

- **[1 Introduction](#introduction)**
- **[2 File name and location](#file)**
- **[3 Default Application](#default)**
- **[4 Scopes](#scopes)**
- **[5 Cache](#cache)**

## 1 Introduction [#](#introduction)

 The freedesktop.org [Desktop Entry specification (http://www.freedesktop.org/wiki/Specifications/desktop-entry-spec/)](http://www.freedesktop.org/wiki/Specifications/desktop-entry-spec/) allows applications to announce which intents they support, using the Implements key.

 An intent can represent the fact that the application provides a certain type of functionality (e.g. "a file manager"), implements a specific D-Bus interface (e.g. `org.freedesktop.FileManager1`), or anything else that is agreed among all applications implementing the intent.

 This specification solves the issue of which application should be used for a given intent by default, and how to let the user set that default application.

## 2 File name and location [#](#file)

 Users, system administrators, application vendors and distributions can change associations between applications and intents by writing into a file called `intentapps.list`.

 The lookup order for this file is as follows:

| `*$XDG_CONFIG_HOME*/*$desktop*-intentapps.list` | user overrides, desktop-specific (for advanced users) |
| --- | --- |
| `*$XDG_CONFIG_HOME*/intentapps.list` | user overrides (recommended location for user configuration GUIs) |
| `*$XDG_CONFIG_DIRS*/*$desktop*-intentapps.list` | sysadmin and ISV overrides, desktop-specific |
| `*$XDG_CONFIG_DIRS*/intentapps.list` | sysadmin and ISV overrides |
| `*$XDG_DATA_DIRS*/applications/*$desktop*-intentapps.list` | distribution-provided defaults, desktop-specific |
| `*$XDG_DATA_DIRS*/applications/intentapps.list` | distribution-provided defaults |

 In this table, *$desktop* is one of the names of the current desktop, lowercase (for instance: `kde`, `gnome`, `xfce`, etc.)

 This is determined from taking the ASCII-lowercase form of a component of the environment variable `$XDG_CURRENT_DESKTOP`, which is a colon-separated list of names that the current desktop is known as.

 The *$desktop* variable should be each of these values in turn.

 All of the above files are referred to as `intentapps.list` in the rest of this specification, for simplicity.

 Note that, despite the similarity to the mimeapps.list spec, it is not possible to add or remove associations from any of these files.

## 3 Default Application [#](#default)

 Indicating the default application for a given intent is done by writing into the group `[Default Applications]` in the file `intentapps.list`.

 The `[Default Applications]` group indicates the default application to be used for a given intent. If the application is no longer installed, the next application in the list is attempted, and so on.

 This example ensures that the application `org.kde.kcalc.desktop` will be used for the `com.example.Calculator1` intent, if it's installed and associated with the intent, and otherwise `org.gnome.Calculator.desktop` if it's installed and associated; and finally falling back to `xcalc.desktop`:

```

  [Default Applications]
  com.example.Calculator1=org.kde.kcalc.desktop;org.gnome.Calculator.desktop;xcalc.desktop;
```

 The value is a semicolon-separated list of desktop file IDs (as defined in the [Desktop Entry Specification (https://specifications.freedesktop.org/desktop-entry-spec/latest/)](https://specifications.freedesktop.org/desktop-entry-spec/latest/) ).

 In the absence of such an entry, the next `intentapps.list` is checked. Once all levels have been checked, if no entry could be found, the implementation should pick the most-preferred `.desktop` files associated with the intent.

 The suggested algorithm for determining the default application for a given intent is:

-

 get the list of desktop IDs for the given intent under the `[Default Applications]` group in the first `intentapps.list`

-

 for each desktop ID in the list, attempt to load the named desktop file, using the normal rules

-

 if a valid desktop file is found, verify that it is associated with the intent

-

 if a valid association is found, we have found the default application

-

 if after all list items are handled, we have not yet found a default application, proceed to the next `intentapps.list` file in the search order and repeat

-

 if, after all files are handled, we have not yet found a default application, we are free to choose how to pick one among the available implementations of the intent. This is implementation defined. It might include hard-coding a preferred default (e.g. from the same desktop environment / software stack), sorting available desktop files by desktop ID to pick the first, or asking the user about their preference. In any case it should be deterministic (for example, not depending on the unsorted enumeration order of files in a directory).

 Note that a desktop ID set as the default for an application can refer to a desktop file of the same name found in a directory of higher precedence.

 Note as well that the default application for a given intent must be an application that is associated with the intent. It is not possible to arbitrarily associate applications with new intents.

 It's also possible to put several implementations of an intent in order of preference by reading all `intentapps.list` files in the order above. Remember however that these files cannot be used to find all implementations of an intent; the desktop files are the canonical source of that information.

## 4 Scopes [#](#scopes)

 It sometimes is not enough to have a single preference order for an intent. For example, an intent which handles opening URIs by scheme (e.g. `http`, `ssh`) might want to define an order for each scheme. To allow this, the desktop file can contain a group with the same name as the interface, where the key `Supports` lists all supported scopes the app supports for the intent (see [Interfaces (https://specifications.freedesktop.org/desktop-entry-spec/latest/interfaces.html)](https://specifications.freedesktop.org/desktop-entry-spec/latest/interfaces.html) from the Desktop Entry Specification).

```

      [Desktop Entry]
      Encoding=UTF-8
      Version=1.0
      Type=Application
      Exec=website-archiver %U
      Name=My Website Archiver
      Implements=com.example.SchemeHandler;

      [com.example.SchemeHandler]
      Supports=https;http;

```

 Equally, every intent that is listed in the `[Default Applications]` group of an `intentapps.list` file can also have its own group where the keys represent the scopes and the values are semicolon-separated lists of desktop file IDs.

 Those lists in the `intentapps.list` files can be used in the same way as above, to find the order of preference for a scope of an intent.

```

  [Default Applications]
  com.example.SchemeHandler=org.mozilla.firefox.desktop;org.gnome.Calculator.desktop;org.gnome.Epiphany.desktop;

  [com.example.SchemeHandler]
  http=org.gnome.Epiphany.desktop;org.mozilla.firefox.desktop;
```

 In this example, the default application for the `http` scope of the `com.example.SchemeHandler` intent is `org.gnome.Epiphany.desktop` (ignoring other files).

## 5 Cache [#](#cache)

 Implementations might want to find out which applications support a specific intent. To make parsing of all the desktop files less costly, an `update-desktop-database` program is provided that will generate a cache file, for applications to avoid reading in (potentially) hundreds of files. It will need to be run after every desktop file is installed. One cache file is created for every directory in `*$XDG_DATA_DIRS*/applications/`, and will create a file called `*$XDG_DATA_DIRS*/applications/intent.cache`.

 The format of the `intent.cache` file is a list mapping intents to desktop files in the `[Intent Cache]` group, where the keys represent the intents and the values are semicolon-separated up-to-date lists of desktop file IDs, representing all applications in this directory which support a particular intent. The list should be sorted by desktop file ID names.

 Additionally, every intent that is listed in the `[Intent Cache]` group can also have its own group where the keys represent the scopes and the values are semicolon-separated lists of desktop file IDs.

 Each intent is listed only once per cache file, and the desktop file IDs are expected to exist in that particular directory. That is to say, if the cache file is located at `/usr/share/applications/intent.cache`, `bar.desktop` refers to the file `/usr/share/applications/bar.desktop`.

 Here is a simple example of an intent cache database:

```

      [Intent Cache]
      com.example.SchemeHandler=org.gnome.Calculator.desktop;org.gnome.Epiphany.desktop;org.mozilla.firefox.desktop;

      [com.example.SchemeHandler]
      http=org.gnome.Epiphany.desktop;org.mozilla.firefox.desktop;

```

© 2026 Freedesktop.org
