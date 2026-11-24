[Jump to content](#_content)[Sound Naming Specification](#index)[Show Contents: Sound Naming Specification](#index)[Top](#)
1. [1 Overview](#overview)
2. [2 Context](#context)
3. [3 Sound Naming Guidelines](#guidelines)
4. [4 Standard Sounds Names](#names)
5. [5 Background](#background)
6. [A Change History](#changelog)

## Sound Naming Specification [#](#index)

Authors: Marc-Andre Lureau `<marcandre.lureau@gmail.com>`Lennart Poettering `<lennart@poettering.net>`Rodney Dawes `<dobey.pwns@gmail.com>`Jon Bold `<jon.wordescapes@co.uk>`
Publication Date: 4 June 2008, Version: Version 0.2

- **[1 Overview](#overview)**
- **[2 Context](#context)**
- **[3 Sound Naming Guidelines](#guidelines)**
- **[4 Standard Sounds Names](#names)**
- **[5 Background](#background)**
- **[A Change History](#changelog)**

## 1 Overview [#](#overview)

 This specification gives direction on how to name the sounds that are available for use by applications, when creating a sound theme. It does so by laying out a standard naming scheme for sound creation, as well as providing a minimal list of must have sounds, and a larger list with many more examples to help with the creation of extended sounds for third party applications, with different event types and usage.

## 2 Context [#](#context)

 The list of default contexts for the sound theme are:

###### Table 1: Standard Contexts [#](#id-1.3.3)

| Name | Description | Directory |
| --- | --- | --- |
| Alerts | Sounds to alert the user of an action or event which may have a major impact on the system or their current use, such as dialog-error. | alert |
| Notifications | Sounds to notify the user that the system, or their current use case has changed state in some way, e.g. new email arriving, new non-critical update available... | notification |
| Actions | Sounds that give the user feedback on their actions. | action |
| Input Feedback | Sounds that give direct response to input events from the user, such as mouse clicks or key presses. | action |
| Game | Sound used in games. | game |

## 3 Sound Naming Guidelines [#](#guidelines)

 Here we define some guidelines for when creating new sounds that extend the standardized list of sound names defined here, in order to provide sounds for more specific events and usages.

-

 Sound names are in the en_US.US_ASCII locale. This means that the characters allowed in the sound names must fall within the US-ASCII character set. As a further restriction, all sound names may only contain lowercase letters, numbers, underscore, dash, or period characters. Spaces, colons, slashes, and backslashes are not allowed. Also, sound names must be spelled as they are in the en_US dictionary.

-

 The dash “-” character is used to separate levels of specificity in sound names. For instance, we use “search-results” as the generic item for all search results, and we use “search-results-empty” for an empty search result. However, if the more specific item does not exist in the current theme, and does exist in a parent theme, the generic sound from the current theme is preferred, in order to keep consistent style. From left to right the words in a sound name become more specific. In some cases what word in a name is more specific is ambiguous. (i.e. "dialog-error" and "error-dialog" both make sense, the former would ease defining the same sound for all dialogs popping up, regardless of its context, the latter would ease defining the same sound for all errors, regardless of how it is presented to the user). In such cases it is generally preferred to put the UI element noun left -- if there is one --, however exceptions of this rule are acceptable.

-

 Sounds for branded applications should be named the same as the binary executable for the application, prefixed by the string “x-”, to avoid name space clashes with future standardized names. Example: “x-openoffice-foobar”.

-

 Please send suggestions for new standardized names to the XDG mailing list: <[xdg@lists.freedesktop.org](mailto:xdg@lists.freedesktop.org)>

## 4 Standard Sounds Names [#](#names)

 This section describes the standard sounds names that should be used by artists when creating themes, and by developers when writing applications which will use the Sound Theme Specification.

### 4.1 Alerts [#](#alerts)

 This is to notify the user of an action or event which may have a major impact on the system or their current use case.

###### Table 2: Standard Alert Sounds [#](#alerts-table)

| Name | Description |
| --- | --- |
| network-connectivity-lost | The sound used when network connectivity is lost. |
| network-connectivity-error | The sound used when an error occurs when it is tried to initialize the network connection of the computing device. |
| dialog-error | The sound used when a dialog is opened to explain an error condition to the user. |
| battery-low | The sound used when the battery is low (below 20%, for example). |
| suspend-error | The machine failed to suspend. |
| software-update-urgent | The sound used when an urgent update is available through the system software update program. |
| power-unplug-battery-low | The power cable has been unplugged and the battery level is low. |

### 4.2 Notifications [#](#notification)

 This is to alert the user that the system, or their current use case has changed state in some way - mew email arriving, new non-critical update to an application available.

###### Table 3: Standard Notifications Sounds [#](#notifications-table)

| Name | Description |
| --- | --- |
| message-new-instant | The sound used when a new IM is recieved. |
| message-new-email | The sound used when a new email is recieved. |
| complete-media-burn | The sound used when an optical medium completed burning. |
| complete-media-rip | The sound used when an optical medium completed ripping. |
| complete-download | The sound used when a file completed downloading. |
| complete-copy | The sound used when a file completed copying. |
| complete-scan | The sound used when a scanner completed image acquisition and software completed image processing. |
| phone-incoming-call | The sound used when an phone/voip call is coming in. Usually some kind of ring sound. |
| phone-outgoing-busy | The sound used when a phone/voip call is dialed out and the responder is not available. |
| phone-hangup | The sound used when a phone/voip call is ended due to hangup. |
| phone-failure | The sound used when a phone/voip call is canceled due to some error. |
| network-connectivity-established | The sound used when network connectivity is established. |
| system-bootup | The sound used when the computer is being booted up, played as as early in bootup as possible. |
| system-ready | The sound used when the computer is booted up and shows the login screen. |
| system-shutdown | The sound used when the computer is being shut down. |
| search-results | The sound used when one or more search results are returned. |
| search-results-empty | The sound used when no search results are returned. |
| desktop-login | The sound used when a user logs into the system, played as a welcome sound immediately after the login screen disappeared. |
| desktop-logout | The sound used when a user logs out of the system. |
| desktop-screen-lock | The sound used when the user locks their current session. |
| service-login | The sound used when a user logs into a service (i.e. Gaim login) |
| service-logout | The sound used when a user logs out of a service (i.e. Gaim logout) |
| battery-caution | The sound used when the battery is nearing exhaustion (below 40%, for example) |
| battery-full | The sound used when the battery is fully loaded up. |
| dialog-warning | The sound used when a dialog is opened to give information to the user that may be pertinent to the requested action. |
| dialog-information | The sound used when a dialog is opened to give information to the user that may be pertinent to the requested action. |
| dialog-question | The sound used when a dialog is opened to ask the user a question. |
| software-update-available | The sound used when an update is available for software installed on the computing device, through the system software update program. |
| device-added | The sound used when a device has become available to the desktop, i.e. due to USB plugging. |
| device-removed | The sound used when a device has become unavailable to the desktop, i.e. due to USB unplug. |
| window-new | The sound used when a new window or dialog is opened. |
| power-plug | The power cable has been plugged in. |
| power-unplug | The power cable has been unplugged. |
| suspend-start | The machine is about to suspend. |
| suspend-resume | The machine has returned from suspended state. |
| lid-open | The lid has been opened (for laptops, mobile devices) |
| lid-close | The lid has been closed (for laptops, mobile devices) |
| alarm-clock-elapsed | A user configured alarm elapsed. |
| window-attention-active | An active/visible window demands attention. |
| window-attention-inactive | An inactive/invisible window demands attention. |

### 4.3 Actions [#](#actions)

 Action sounds are used as feedback for user operations.

###### Table 4: Standard Action Sounds [#](#actions-table)

| Name | Description |
| --- | --- |
| phone-outgoing-calling | The sound used when a phone/voip call is dialed out. |
| message-sent-instant | The sound used when a new IM is sent. |
| message-sent-email | The sound used when a new email is sent. |
| bell-terminal | The sound to use as a terminal bell. |
| bell-window-system | The sound to use as a generic bell for X11 or other window systems. |
| trash-empty | The sound used when the user empties the trash. |
| item-deleted | The sound used when a item is deleted. |
| file-trash | The sound used when a file or folder is sent to the trash. |
| camera-shutter | A photo has been shot. |
| camera-focus | A camera has the focus. |
| screen-capture | A screenshot was made. |
| count-down | A countdown (e.g. for a photo shooting) sound that is repeated each second. |
| completion-sucess | A text completion was attempted and was successful. |
| completion-fail | A text completion was attempted and was not successful. |
| completion-partial | A text completion was attempted and was partially successful. |
| completion-rotation | A text completion was attempted and the list of available options reached the end and completion started from the beginning. |
| audio-volume-change | The test sound that is used to make volume changes noticeable by the user when he uses a volume slider. |
| audio-channel-left | The test sound for identifying the left speaker. A mono file with a voice saying "left". |
| audio-channel-right | The test sound for identifying the right speaker. A mono file with a voice saying "right". |
| audio-channel-front-left | The test sound for identifying the front-left speaker. A mono file with a voice saying "front left". |
| audio-channel-front-right | The test sound for identifying the front-right speaker. A mono file with a voice saying "front right". |
| audio-channel-front-center | The test sound for identifying the front-center speaker. A mono file with a voice saying "front center". |
| audio-channel-rear-left | The test sound for identifying the front-left speaker. A mono file with a voice saying "rear left". |
| audio-channel-rear-right | The test sound for identifying the front-right speaker. A mono file with a voice saying "rear right". |
| audio-channel-rear-center | The test sound for identifying the front-center speaker. A mono file with a voice saying "rear center". |
| audio-channel-lfe | The test sound for identifying the lfe/subwoofer speaker. A mono file with a voice saying "subwoofer", alternatively a low frequency noise. |
| audio-channel-side-left | The test sound for identifying the side-left speaker. A mono file with a voice saying "side left". |
| audio-channel-side-right | The test sound for identifying the side-right speaker. A mono file with a voice saying "side right". |
| audio-test-signal | The test sound for testing audio. |

### 4.4 Input Feedback [#](#inputfeedback)

 Actions sounds are used as feedback for user input events, such as mouse clicks, or key presses. In contrast to the sounds listed as "Actions" these sounds contain much less context information are are solely used to give the user audible feedback to input events the user himself directly caused.

###### Table 5: Standard Input Feedback Sounds [#](#inputfeedback-table)

| Name | Description |
| --- | --- |
| window-close | The sound used when an existing window is closed. |
| window-slide-in | The sound used when a window is slided in by some means. Example: panel. |
| window-slide-out | The sound used when a window is slided out by some means. Example: panel. |
| window-minimized | The sound used when an existing window is minimized. |
| window-unminimized | The sound used when an existing window is unminimized. |
| window-maximized | The sound used when an existing window is maximized. |
| window-unmaximized | The sound used when an existing window is unmaximized. |
| window-inactive-click | The sound used when the user clicks on an inactive window. |
| window-move-start | A window move started. |
| window-move-end | A window move ended. |
| window-resize-start | A window resize started. |
| window-resize-end | A window resize ended. |
| desktop-switch-left | The sound used when the window manager switches to another desktop which is located to the left of the current screen. |
| desktop-switch-right | The sound used when the window manager switches to another desktop which is located to the right of the current screen. |
| window-switch | The sound used when the window manager switches to another window. |
| notebook-tab-changed | The sound used when a notebook tab is changed. |
| scroll-up | Some window was scrolled up. |
| scroll-down | Some window was scrolled down. |
| scroll-left | Some window was scrolled left. |
| scroll-right | Some window was scrolled right. |
| scroll-up-end | Some window was scrolled up and reached the end of the scrollbar. |
| scroll-down-end | Some window was scrolled down and reached the end of the scrollbar. |
| scroll-left-end | Some window was scrolled left and reached the end of the scrollbar. |
| scroll-right-end | Some window was scrolled right and reached the end of the scrollbar. |
| dialog-ok | The sound used when a windows is closed by clicking on the OK button for a window. |
| dialog-cancel | The sound used when a windows is closed by clicking on the Cancel button for a window. |
| drag-start | The sound used when drag of a file/item is started. |
| drag-accept | The sound used when a file/item drag is accepted by a window, such as a folder or IM conversation. |
| drag-fail | The sound used when a file/item drag is not accepted by a window, such as a folder or IM conversation. |
| link-pressed | The sound used when a link in a web or help browser is pressed. |
| link-released | The sound used when a link in a web or help browser is releaed. |
| button-pressed | The sound used when a button is pressed. |
| button-released | The sound used when a button is released. |
| menu-click | The sound used when a menu item is clicked. |
| button-toggle-on | The sound used when a toggle/check/radio button is activated. |
| button-toggle-off | The sound used when a toggle/check/radio button is deactivated. |
| expander-toggle-on | The sound used when an expander is activated. |
| expander-toggle-off | The sound used when an expander is deactivated. |
| menu-popup | The sound used when a menu is popped up. |
| menu-popdown | The sound used when a menu is popped down. |
| menu-replace | The sound used when replacing an active menu with another menu. |
| tooltip-popup | The sound used when a tooltip is popped up. |
| tooltip-popdown | The sound used when a tooltip is popped down. |
| item-selected | The sound used when an item is selected. |

### 4.5 Game [#](#games)

 Sounds for usage in games.

###### Table 6: Standard Games Sounds [#](#games-table)

| Name | Description |
| --- | --- |
| game-over-winner | Guess what...! |
| game-over-loser | Guess what...! |
| game-card-shuffle | In card games, when the cards are shuffled. |
| game-human-move | When the user makes a move. |
| game-computer-move | When the a computer makes a move. |

## 5 Background [#](#background)

 The sound naming specification is heavily based on the "Icon Naming Specification" by Rodney Dawes and the "Bango Project" started by Jon Bolt.

## A Change History [#](#changelog)

Version 0.3, 25 July 2008, Lennart Poettering.

-

 Split out section about "Input Feedback" sounds.

-

 Document the logic behind choosing "dialog-error" instead of "error-dialog".

Version 0.2, 4 June 2008, Marc-Andre Lureau and Lennart Poettering.

-

 Naming changes.

Version 0.1, February 12 2008, Marc-Andre Lureau.

-

 Created initial draft.

© 2026 Freedesktop.org
