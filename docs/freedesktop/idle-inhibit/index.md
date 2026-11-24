[Jump to content](#_content)[Idle Inhibition Service Draft](#index)[Show Contents: Idle Inhibition Service Draft](#index)[Top](#)
1. [I API Documentation](#description)
  1. [1 Introduction](#introduction)
  2. [2 Consequences of idleness](#consequences)
  3. [3 API overview](#api)
  4. [4 Design notes](#id-1.2.5)

2. [II D-Bus API Reference](#ref-dbus-api)
  1. [5 Interfaces](#interfaces)

3. [III References](#id-1.4)
  1. [6 References](#id-1.4.2)

# Idle Inhibition Service Draft

Author: Bastien Nocera `<hadess@hadess.net>`(GNOME core Developer)
Publication Date: 2026-11-20, Version:  Idle Inhibition Service 0.1 DRAFT

Copyright © 2012 The Idle Inhibition Service API Authors

- **[I API Documentation](#description)**

- **[1 Introduction](#introduction)**
- **[2 Consequences of idleness](#consequences)**
- **[3 API overview](#api)**
- **[4 Design notes](#id-1.2.5)**

- **[II D-Bus API Reference](#ref-dbus-api)**

- **[5 Interfaces](#interfaces)**

- **[`org.freedesktop.ScreenSaver`](#org.freedesktop.ScreenSaver) — The Idle Inhibition Service manages the inhibition requests. **

- **[III References](#id-1.4)**

- **[6 References](#id-1.4.2)**

# Part I API Documentation [#](#description)

- **[1 Introduction](#introduction)**

The Idle Inhibition Service API allows client applications to inhibit a user's login session from becoming idle.

- **[2 Consequences of idleness](#consequences)**

Depending on the user session software running, idleness can have multiple consequences, including, but not limited to:
-

Screen backlight dimming

-

Screensaver animation replacing the desktop content

-

Instant messenging client going to "auto-away"

-

Computer suspending or hibernating

- **[3 API overview](#api)**

Idle inhibition is achieved by the application calling an [`Inhibit`](#org.freedesktop.ScreenSaver.Inhibit) method on a well-known D-Bus name.

- **[4 Design notes](#id-1.2.5)**

The D-Bus service name, org.freedesktop.ScreenSaver, intentionally contains the word screensaver, as KDE implements the idle inhibition API in a screensaver sub-module. It was necessary for the specified Idle Inhibition Service API to be compatible with this existing software.

## 1 Introduction [#](#introduction)

The Idle Inhibition Service API allows client applications to inhibit a user's login session from becoming idle.

The service is usually implemented by the session manager, or another daemon running inside the user's session, and communicating with the platform software to avoid becoming idle, and the consequences of becoming idle.

This API was designed by GNOME and KDE developers with the goal of having a common way to inhibit idleness.

## 2 Consequences of idleness [#](#consequences)

Depending on the user session software running, idleness can have multiple consequences, including, but not limited to:

-

Screen backlight dimming

-

Screensaver animation replacing the desktop content

-

Instant messenging client going to "auto-away"

-

Computer suspending or hibernating

Inhibiting idleness is supposed to stop all those actions from taking place, specifically in response to idleness. A user action asking for any of those states would obviously be acted upon witout delay.

## 3 API overview [#](#api)

Idle inhibition is achieved by the application calling an [`Inhibit`](#org.freedesktop.ScreenSaver.Inhibit) method on a well-known D-Bus name.

Inhibition will stop when the [` UnInhibit`](#org.freedesktop.ScreenSaver.UnInhibit) method is called, or the application disconnects from the D-Bus session bus (which usually happens upon exit).

Implementations of this well-known bus name must have an object /org/freedesktop/ScreenSaver which implements the [org.freedesktop.ScreenSaver](#org.freedesktop.ScreenSaver) interface.

## 4 Design notes [#](#id-1.2.5)

The D-Bus service name, org.freedesktop.ScreenSaver, intentionally contains the word screensaver, as KDE implements the idle inhibition API in a screensaver sub-module. It was necessary for the specified Idle Inhibition Service API to be compatible with this existing software.

KDE's implementation until kde-workspace 4.11.8 lived on the /ScreenSaver object, not on /org/freedesktop/ScreenSaver as described in this specification. This is fixed in later releases.

For the same reasons, inhibition of suspend, hibernation, or user-switching (amongst others) is not supported in the API.

# Part II D-Bus API Reference [#](#ref-dbus-api)

- **[5 Interfaces](#interfaces)**

Inhibit idleness for the caller application.

## 5 Interfaces [#](#interfaces)

- **[`org.freedesktop.ScreenSaver`](#org.freedesktop.ScreenSaver) — The Idle Inhibition Service manages the inhibition requests. **

## `org.freedesktop.ScreenSaver`

org.freedesktop.ScreenSaver — The Idle Inhibition Service manages the inhibition requests.

## Synopsis

### Methods

| `**Inhibit** (` | IN String `application_name`, |
| --- | --- |
|  | IN String `reason_for_inhibit`, |
|  | OUT UInt32 `cookie``)`; |

| `**UnInhibit** (` | IN UInt32 `cookie``)`; |
| --- | --- |

## Methods

### `org.freedesktop.ScreenSaver.Inhibit`

| `**Inhibit** (` | IN String `application_name`, |
| --- | --- |
|  | IN String `reason_for_inhibit`, |
|  | OUT UInt32 `cookie``)`; |

Inhibit idleness for the caller application.

- **`application_name`[#](#id-1.3.2.2.4.2.4.1)**

A unique identifier for the application, usually a reverse domain (such as 'org.freedesktop.example').

- **`reason_for_inhibit`[#](#id-1.3.2.2.4.2.4.2)**

A human-readable and possibly translated string explaining the reason why idleness is inhibited (such as 'Playing a movie').

- **`cookie`[#](#id-1.3.2.2.4.2.4.3)**

A cookie uniquely representing the inhibition request, to be passed to UnInhibit when done.

### `org.freedesktop.ScreenSaver.UnInhibit`

| `**UnInhibit** (` | IN UInt32 `cookie``)`; |
| --- | --- |

Disable inhibit idleness for the caller application.

- **`cookie`[#](#id-1.3.2.2.4.3.4.1)**

A cookie representing the inhibition request, as returned by the 'Inhibit' function.

# Part III References [#](#id-1.4)

- **[6 References](#id-1.4.2)**

[KDE Screenlocker D-Bus API (https://projects.kde.org/projects/kde/kde-workspace/repository/revisions/master/entry/ksmserver/screenlocker/dbus/org.freedesktop.ScreenSaver.xml)](https://projects.kde.org/projects/kde/kde-workspace/repository/revisions/master/entry/ksmserver/screenlocker/dbus/org.freedesktop.ScreenSaver.xml)

## 6 References [#](#id-1.4.2)

- **kde-api**

[KDE Screenlocker D-Bus API (https://projects.kde.org/projects/kde/kde-workspace/repository/revisions/master/entry/ksmserver/screenlocker/dbus/org.freedesktop.ScreenSaver.xml)](https://projects.kde.org/projects/kde/kde-workspace/repository/revisions/master/entry/ksmserver/screenlocker/dbus/org.freedesktop.ScreenSaver.xml)

- **gnome-impl**

[GNOME org.freedesktop.ScreenSaver proxy implementation (http://git.gnome.org/browse/gnome-settings-daemon/tree/plugins/screensaver-proxy)](http://git.gnome.org/browse/gnome-settings-daemon/tree/plugins/screensaver-proxy)

© 2026 Freedesktop.org
