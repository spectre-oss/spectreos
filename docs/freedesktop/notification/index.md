[Jump to content](#_content)[Desktop Notifications Specification](#index)[Show Contents: Desktop Notifications Specification](#index)[Top](#)
1. [1 Introduction](#introduction)
2. [2 Basic Design](#basic-design)
3. [3 Backwards Compatibility](#backwards-compat)
4. [4 Markup](#markup)
5. [5 Icons and Images](#icons-and-images)
6. [6 Categories](#categories)
7. [7 Urgency Levels](#urgency-levels)
8. [8 Hints](#hints)
9. [9 D-Bus Protocol](#protocol)

## Desktop Notifications Specification [#](#index)

Authors: Mike Hearn `<mike@navi.cx>`Christian Hammond `<chipx86@chipx86.com>`William Jon McCann `<jmccann@redhat.com>`
Publication Date: 18 August 2024, Version: Version 1.3

- **[1 Introduction](#introduction)**
- **[2 Basic Design](#basic-design)**
- **[3 Backwards Compatibility](#backwards-compat)**
- **[4 Markup](#markup)**
- **[5 Icons and Images](#icons-and-images)**
- **[6 Categories](#categories)**
- **[7 Urgency Levels](#urgency-levels)**
- **[8 Hints](#hints)**
- **[9 D-Bus Protocol](#protocol)**

## 1 Introduction [#](#introduction)

 This is a standard for a desktop notifications service, through which applications can generate passive popups to notify the user in an asynchronous manner of events.

 This specification explicitly does not include other types of notification presentation such as modal message boxes, window manager decorations or window list annotations.

 Example use cases include:

-

Messages from chat programs

-

Scheduled alarm

-

Completed file transfer

-

New mail notification

-

Low disk space/battery warnings

## 2 Basic Design [#](#basic-design)

 In order to ensure that multiple notifications can easily be displayed at once, and to provide a convenient implementation, all notifications are controlled by a single session-scoped service which exposes a D-Bus interface.

 On startup, a conforming implementation should take the `org.freedesktop.Notifications` service on the session bus. This service will be referred to as the "notification server" or just "the server" in this document. It can optionally be activated automatically by the bus process, however this is not required and notification server clients must not assume that it is available.

 The server should implement the `org.freedesktop.Notifications` interface on an object with the path `"/org/freedesktop/Notifications"`. This is the only interface required by this version of the specification.

 A notification has the following components:

###### Table 1: Notification Components [#](#id-1.3.6)

| Component | Description |
| --- | --- |
| Application Name | This is the optional name of the application sending the notification. This should be the application's formal name, rather than some sort of ID. An example would be "FredApp E-Mail Client," rather than "fredapp-email-client." |
| Replaces ID | An optional ID of an existing notification that this notification is intended to replace. |
| Notification Icon | The notification icon. See [Icons and Images Formats](#icons-and-images-formats). |
| Summary | This is a single line overview of the notification. For instance, "You have mail" or "A friend has come online". It should generally not be longer than 40 characters, though this is not a requirement, and server implementations should word wrap if necessary. The summary must be encoded using UTF-8. |
| Body | This is a multi-line body of text. Each line is a paragraph, server implementations are free to word wrap them as they see fit. The body may contain simple markup as specified in [Markup](#markup). It must be encoded using UTF-8. If the body is omitted, just the summary is displayed. |
| Actions | The actions send a request message back to the notification client when invoked. This functionality may not be implemented by the notification server, conforming clients should check if it is available before using it (see the GetCapabilities message in [Protocol](#protocol)). An implementation is free to ignore any requested by the client. As an example one possible rendering of actions would be as buttons in the notification popup. Actions are sent over as a list of pairs. Each even element in the list (starting at index 0) represents the identifier for the action. Each odd element in the list is the localized string that will be displayed to the user. The default action (usually invoked by clicking the notification) should have a key named `"default"`. The name can be anything, though implementations are free not to display it. |
| Hints | Hints are a way to provide extra data to a notification server that the server may be able to make use of. See [Hints](#hints) for a list of available hints. |
| Expiration Timeout | The timeout time in milliseconds since the display of the notification at which the notification should automatically close. If -1, the notification's expiration time is dependent on the notification server's settings, and may vary for the type of notification. If 0, the notification never expires. |

 Each notification displayed is allocated a unique ID by the server. This is unique within the session. While the notification server is running, the ID will not be recycled unless the capacity of a uint32 is exceeded.

 This can be used to hide the notification before the expiration timeout is reached. It can also be used to atomically replace the notification with another. This allows you to (for instance) modify the contents of a notification while it's on-screen.

## 3 Backwards Compatibility [#](#backwards-compat)

 Clients should try and avoid making assumptions about the presentation and abilities of the notification server. The message content is the most important thing.

 Clients can check with the server what capabilities are supported using the `GetCapabilities` message. See [Protocol](#protocol).

 If a client requires a response from a passive popup, it should be coded such that a non-focus-stealing message box can be used in the case that the notification server does not support this feature.

## 4 Markup [#](#markup)

 Body text may contain markup. The markup is XML-based, and consists of a small subset of HTML along with a few additional tags.

 The following tags should be supported by the notification server. Though it is optional, it is recommended. Notification servers that do not support these tags should filter them out.

| `<b>` ... `</b>` | Bold |
| --- | --- |
| `<i>` ... `</i>` | Italic |
| `<u>` ... `</u>` | Underline |
| `<a href="...">` ... `</a>` | Hyperlink |
| `<img src="..." alt="..."/>` | Image |

 A full-blown HTML implementation is not required of this spec, and notifications should never take advantage of tags that are not listed above. As notifications are not a substitute for web browsers or complex dialogs, advanced layout is not necessary, and may in fact limit the number of systems that notification services can run on, due to memory usage and screen space. Such examples are PDAs, certain cell phones, and slow PCs or laptops with little memory.

 For the same reason, a full XML or XHTML implementation using XSLT or CSS stylesheets is not part of this specification. Information that must be presented in a more complex form should use an application-specific dialog, a web browser, or some other display mechanism.

 The tags specified above mark up the content in a way that allows them to be stripped out on some implementations without impacting the actual content.

### 4.1 Hyperlinks [#](#hyperlinks)

 Hyperlinks allow for linking one or more words to a URI. There is no requirement to allow for images to be linked, and it is highly suggested that implementations do not allow this, as there is no clean-looking, standard visual indicator for a hyperlinked image.

 Hyperlinked text should appear in the standard blue underline format.

 Hyperlinks cannot function as a replacement for actions. They are used to link to local directories or remote sites using standard URI schemes.

 Implementations are not required to support hyperlinks.

### 4.2 Images [#](#images)

 Images may be placed in the notification, but this should be done with caution. The image should never exceed 200x100, but this should be thought of as a maximum size. Images should always have alternative text provided through the `alt="..."` attribute.

 Image data cannot be embedded in the message itself. Images referenced must always be local files.

 Implementations are not required to support images.

## 5 Icons and Images [#](#icons-and-images)

 A notification can optionally have an associated icon and/or image.

 The icon is defined by the "app_icon" parameter. The image can be defined by the "image-path", the "image-data" hint or the deprecated "icon_data" hint.

### 5.1 Priorities [#](#id-1.6.4)

 An implementation which only displays one image or icon must choose which one to display using the following order:

1.

"image-data"

2.

"image-path"

3.

app_icon parameter

4.

for compatibility reason, "icon_data"

 An implementation which can display both the image and icon must show the icon from the "app_icon" parameter and choose which image to display using the following order:

1.

"image-data"

2.

"image-path"

3.

for compatibility reason, "icon_data"

### 5.2 Formats [#](#icons-and-images-formats)

 The "image-data" and "icon_data" hints should be a D-Bus structure of signature (iiibiiay). The components of this structure are as follows:

1.

width (i): Width of image in pixels

2.

height (i): Height of image in pixels

3.

rowstride (i): Distance in bytes between row starts

4.

has_alpha (b): Whether the image has an alpha channel

5.

bits_per_sample (i): Must always be 8

6.

channels (i): If has_alpha is TRUE, must be 4, otherwise 3

7.

data (ay): The image data, in RGB byte order

 This image format is derived from [gdk-pixbuf (http://developer.gnome.org/gdk-pixbuf/stable/)](http://developer.gnome.org/gdk-pixbuf/stable/).

 The "app_icon" parameter and "image-path" hint should be either an URI (file:// is the only URI schema supported right now) or a name in a freedesktop.org-compliant icon theme (not a GTK+ stock ID).

## 6 Categories [#](#categories)

 Notifications can optionally have a type indicator. Although neither client or nor server must support this, some may choose to. Those servers implementing categories may use them to intelligently display the notification in a certain way, or group notifications of similar types.

 Categories are in `*class.specific*` form. `class` specifies the generic type of notification, and `specific` specifies the more specific type of notification.

 If a specific type of notification does not exist for your notification, but the generic kind does, a notification of type `*class*` is acceptable.

 Third parties, when defining their own categories, should discuss the possibility of standardizing on the hint with other parties, preferably in a place such as the [xdg (http://freedesktop.org/mailman/listinfo/xdg)](http://freedesktop.org/mailman/listinfo/xdg) mailing list at [freedesktop.org (http://freedesktop.org/)](http://freedesktop.org/). If it warrants a standard, it will be added to the table above. If no consensus is reached, the category should be in the form of "`x-*vendor*.*class*.*name*`."

 The following table lists standard notifications as defined by this spec. More will be added in time.

###### Table 2: Categories [#](#id-1.7.7)

| Type | Description |
| --- | --- |
| `"call"` | A generic audio or video call notification that doesn't fit into any other category. |
| `"call.ended"` | An audio or video call was ended. |
| `"call.incoming"` | A audio or video call is incoming. |
| `"call.unanswered"` | An incoming audio or video call was not answered. |
| `"device"` | A generic device-related notification that doesn't fit into any other category. |
| `"device.added"` | A device, such as a USB device, was added to the system. |
| `"device.error"` | A device had some kind of error. |
| `"device.removed"` | A device, such as a USB device, was removed from the system. |
| `"email"` | A generic e-mail-related notification that doesn't fit into any other category. |
| `"email.arrived"` | A new e-mail notification. |
| `"email.bounced"` | A notification stating that an e-mail has bounced. |
| `"im"` | A generic instant message-related notification that doesn't fit into any other category. |
| `"im.error"` | An instant message error notification. |
| `"im.received"` | A received instant message notification. |
| `"network"` | A generic network notification that doesn't fit into any other category. |
| `"network.connected"` | A network connection notification, such as successful sign-on to a network service. This should not be confused with `device.added` for new network devices. |
| `"network.disconnected"` | A network disconnected notification. This should not be confused with `device.removed` for disconnected network devices. |
| `"network.error"` | A network-related or connection-related error. |
| `"presence"` | A generic presence change notification that doesn't fit into any other category, such as going away or idle. |
| `"presence.offline"` | An offline presence change notification. |
| `"presence.online"` | An online presence change notification. |
| `"transfer"` | A generic file transfer or download notification that doesn't fit into any other category. |
| `"transfer.complete"` | A file transfer or download complete notification. |
| `"transfer.error"` | A file transfer or download error. |

## 7 Urgency Levels [#](#urgency-levels)

 Notifications have an urgency level associated with them. This defines the importance of the notification. For example, "Joe Bob signed on" would be a low urgency. "You have new mail" or "A USB device was unplugged" would be a normal urgency. "Your computer is on fire" would be a critical urgency.

Urgency levels are defined as follows:

###### Table 3: Urgency Levels [#](#id-1.8.4)

| Type | Description |
| --- | --- |
| 0 | Low |
| 1 | Normal |
| 2 | Critical |

 Developers must use their own judgement when deciding the urgency of a notification. Typically, if the majority of programs are using the same level for a specific type of urgency, other applications should follow them.

 For low and normal urgencies, server implementations may display the notifications how they choose. They should, however, have a sane expiration timeout dependent on the urgency level.

 Critical notifications should not automatically expire, as they are things that the user will most likely want to know about. They should only be closed when the user dismisses them, for example, by clicking on the notification.

## 8 Hints [#](#hints)

 Hints are a way to provide extra data to a notification server that the server may be able to make use of.

 Neither clients nor notification servers are required to support any hints. Both sides should assume that hints are not passed, and should ignore any hints they do not understand.

 Third parties, when defining their own hints, should discuss the possibility of standardizing on the hint with other parties, preferably in a place such as the [xdg (http://freedesktop.org/mailman/listinfo/xdg)](http://freedesktop.org/mailman/listinfo/xdg) mailing list at [freedesktop.org (http://freedesktop.org/)](http://freedesktop.org/). If it warrants a standard, it will be added to the table above. If no consensus is reached, the hint name should be in the form of `"x-*vendor*-*name*."`

 The value type for the hint dictionary in D-Bus is of the `DBUS_TYPE_VARIANT` container type. This allows different data types (string, integer, boolean, etc.) to be used for hints. When adding a dictionary of hints, this type must be used, rather than putting the actual hint value in as the dictionary value.

 The following table lists the standard hints as defined by this specification. Future hints may be proposed and added to this list over time. Once again, implementations are not required to support these.

###### Table 4: Standard Hints [#](#id-1.9.7)

| Name | Value Type | Description | Spec Version |
| --- | --- | --- | --- |
| `"action-icons"` | BOOLEAN | When set, a server that has the "action-icons" capability will attempt to interpret any action identifier as a named icon. The localized display name will be used to annotate the icon for accessibility purposes. The icon name should be compliant with the Freedesktop.org Icon Naming Specification. | >= 1.2 |
| `"category"` | STRING | The type of notification this is. |  |
| `"desktop-entry"` | STRING | This specifies the name of the desktop filename representing the calling program. This should be the same as the prefix used for the application's .desktop file. An example would be "rhythmbox" from "rhythmbox.desktop". This can be used by the daemon to retrieve the correct icon for the application, for logging purposes, etc. |  |
| `"image-data"` | (iiibiiay) | This is a raw data image format which describes the width, height, rowstride, has alpha, bits per sample, channels and image data respectively. | >= 1.2 |
| `"image_data"` | (iiibiiay) | *Deprecated*. Use image-data instead. | = 1.1 |
| `"image-path"` | STRING | Alternative way to define the notification image. See [Icons and Images](#icons-and-images). | >= 1.2 |
| `"image_path"` | STRING | *Deprecated*. Use image-path instead. | = 1.1 |
| `"icon_data"` | (iiibiiay) | *Deprecated*. Use image-data instead. | < 1.1 |
| `"resident"` | BOOLEAN | When set the server will not automatically remove the notification when an action has been invoked. The notification will remain resident in the server until it is explicitly removed by the user or by the sender. This hint is likely only useful when the server has the "persistence" capability. | >= 1.2 |
| `"sound-file"` | STRING | The path to a sound file to play when the notification pops up. |  |
| `"sound-name"` | STRING | A themeable named sound from the freedesktop.org [sound naming specification (http://0pointer.de/public/sound-naming-spec.html)](http://0pointer.de/public/sound-naming-spec.html) to play when the notification pops up. Similar to icon-name, only for sounds. An example would be "message-new-instant". |  |
| `"suppress-sound"` | BOOLEAN | Causes the server to suppress playing any sounds, if it has that ability. This is usually set when the client itself is going to play its own sound. |  |
| `"transient"` | BOOLEAN | When set the server will treat the notification as transient and by-pass the server's persistence capability, if it should exist. | >= 1.2 |
| `"x"` | INT32 | Specifies the X location on the screen that the notification should point to. The `"y"` hint must also be specified. |  |
| `"y"` | INT32 | Specifies the Y location on the screen that the notification should point to. The `"x"` hint must also be specified. |  |
| `"urgency"` | BYTE | The urgency level. |  |

## 9 D-Bus Protocol [#](#protocol)

 The following messages *must* be supported by all implementations.

### 9.1 Message commands [#](#commands)

#### 9.1.1 `org.freedesktop.Notifications.GetCapabilities`[#](#command-get-capabilities)

| `as **org.freedesktop.Notifications.GetCapabilities** (` | `void)`; |  |
| --- | --- | --- |

 This message takes no parameters.

 It returns an array of strings. Each string describes an optional capability implemented by the server. The following values are defined by this spec:

###### Table 5: Server Capabilities [#](#id-1.10.3.2.5)

| `"action-icons"` | Supports using icons instead of text for displaying actions. Using icons for actions must be enabled on a per-notification basis using the "action-icons" hint. |
| --- | --- |
| `"actions"` | The server will provide the specified actions to the user. Even if this cap is missing, actions may still be specified by the client, however the server is free to ignore them. |
| `"body"` | Supports body text. Some implementations may only show the summary (for instance, onscreen displays, marquee/scrollers) |
| `"body-hyperlinks"` | The server supports hyperlinks in the notifications. |
| `"body-images"` | The server supports images in the notifications. |
| `"body-markup"` | Supports markup in the body text. If marked up text is sent to a server that does not give this cap, the markup will show through as regular text so must be stripped clientside. |
| `"icon-multi"` | The server will render an animation of all the frames in a given image array. The client may still specify multiple frames even if this cap and/or `"icon-static"` is missing, however the server is free to ignore them and use only the primary frame. |
| `"icon-static"` | Supports display of exactly 1 frame of any given image array. This value is mutually exclusive with `"icon-multi"`, it is a protocol error for the server to specify both. |
| `"persistence"` | The server supports persistence of notifications. Notifications will be retained until they are acknowledged or removed by the user or recalled by the sender. The presence of this capability allows clients to depend on the server to ensure a notification is seen and eliminate the need for the client to display a reminding function (such as a status icon) of its own. |
| `"sound"` | The server supports sounds on notifications. If returned, the server must support the `"sound-file"` and `"suppress-sound"` hints. |

 New vendor-specific caps may be specified as long as they start with `"x-*vendor*"`. For instance, `"x-gnome-foo-cap"`. Capability names must not contain spaces. They are limited to alpha-numeric characters and dashes (`"-"`).

#### 9.1.2 `org.freedesktop.Notifications.Notify`[#](#command-notify)

| `UINT32 **org.freedesktop.Notifications.Notify** (` | STRING `app_name`, |
| --- | --- |
|  | UINT32 `replaces_id`, |
|  | STRING `app_icon`, |
|  | STRING `summary`, |
|  | STRING `body`, |
|  | as `actions`, |
|  | a{sv} `hints`, |
|  | INT32 `expire_timeout``)`; |

 Sends a notification to the notification server.

###### Table 6: Notify Parameters [#](#id-1.10.3.3.4)

| Name | Type | Description |
| --- | --- | --- |
| *app_name* | STRING | The optional name of the application sending the notification. Can be blank. |
| *replaces_id* | UINT32 | The optional notification ID that this notification replaces. The server must atomically (ie with no flicker or other visual cues) replace the given notification with this one. This allows clients to effectively modify the notification while it's active. A value of value of 0 means that this notification won't replace any existing notifications. |
| *app_icon* | STRING | The optional program icon of the calling application. See [Icons and Images](#icons-and-images). Can be an empty string, indicating no icon. |
| *summary* | STRING | The summary text briefly describing the notification. |
| *body* | STRING | The optional detailed body text. Can be empty. |
| *actions* | as | Actions are sent over as a list of pairs. Each even element in the list (starting at index 0) represents the identifier for the action. Each odd element in the list is the localized string that will be displayed to the user. |
| *hints* | a{sv} | Optional hints that can be passed to the server from the client program. Although clients and servers should never assume each other supports any specific hints, they can be used to pass along information, such as the process PID or window ID, that the server may be able to make use of. See [Hints](#hints). Can be empty. |
| *expire_timeout* | INT32 | The timeout time in milliseconds since the display of the notification at which the notification should automatically close. If -1, the notification's expiration time is dependent on the notification server's settings, and may vary for the type of notification. If 0, never expire. |

 If *replaces_id* is 0, the return value is a UINT32 that represent the notification. It is unique, and will not be reused unless a `MAXINT` number of notifications have been generated. An acceptable implementation may just use an incrementing counter for the ID. The returned ID is always greater than zero. Servers must make sure not to return zero as an ID.

 If *replaces_id* is not 0, the returned value is the same value as *replaces_id*.

#### 9.1.3 `org.freedesktop.Notifications.CloseNotification`[#](#command-close-notification)

| `void **org.freedesktop.Notifications.CloseNotification** (` | UINT32 `id``)`; |
| --- | --- |

 Causes a notification to be forcefully closed and removed from the user's view. It can be used, for example, in the event that what the notification pertains to is no longer relevant, or to cancel a notification with no expiration time.

 The `NotificationClosed` signal is emitted by this method.

 If the notification no longer exists, an empty D-Bus Error message is sent back.

#### 9.1.4 `org.freedesktop.Notifications.GetServerInformation`[#](#command-get-server-information)

| ` void **org.freedesktop.Notifications.GetServerInformation** (` | out STRING `name`, |
| --- | --- |
|  | out STRING `vendor`, |
|  | out STRING `version`, |
|  | out STRING `spec_version``)`; |

 This message returns the information on the server. Specifically, the server name, vendor, and version number.

###### Table 7: GetServerInformation Return Values [#](#id-1.10.3.5.4)

| Name | Type | Description |
| --- | --- | --- |
| *name* | STRING | The product name of the server. |
| *vendor* | STRING | The vendor name. For example, "KDE," "GNOME," "freedesktop.org," or "Microsoft." |
| *version* | STRING | The server's version number. |
| *spec_version* | STRING | The specification version the server is compliant with. |

### 9.2 Signals [#](#signals)

#### 9.2.1 `org.freedesktop.Notifications.NotificationClosed`[#](#signal-notification-closed)

| `**org.freedesktop.Notifications.NotificationClosed** (` | UINT32 `id`, |
| --- | --- |
|  | UINT32 `reason``)`; |

 A completed notification is one that has timed out, or has been dismissed by the user.

###### Table 8: NotificationClosed Parameters [#](#id-1.10.4.2.4)

| Name | Type | Description |
| --- | --- | --- |
| *id* | UINT32 | The ID of the notification that was closed. |
| *reason* | UINT32 | The reason the notification was closed. 1 - The notification expired. 2 - The notification was dismissed by the user. 3 - The notification was closed by a call to `CloseNotification`. 4 - Undefined/reserved reasons. |

 The ID specified in the signal is invalidated *before* the signal is sent and may not be used in any further communications with the server.

#### 9.2.2 `org.freedesktop.Notifications.ActionInvoked`[#](#signal-action-invoked)

| `**org.freedesktop.Notifications.ActionInvoked** (` | UINT32 `id`, |
| --- | --- |
|  | STRING `action_key``)`; |

 This signal is emitted when one of the following occurs:

-

 The user performs some global "invoking" action upon a notification. For instance, clicking somewhere on the notification itself.

-

 The user invokes a specific action as specified in the original Notify request. For example, clicking on an action button.

###### Table 9: ActionInvoked Parameters [#](#id-1.10.4.3.5)

| Name | Type | Description |
| --- | --- | --- |
| *id* | UINT32 | The ID of the notification emitting the ActionInvoked signal. |
| *action_key* | STRING | The key of the action invoked. These match the keys sent over in the list of actions. |

###### Note

 Clients should not assume the server will generate this signal. Some servers may not support user interaction at all, or may not support the concept of being able to "invoke" a notification.

#### 9.2.3 `org.freedesktop.Notifications.ActivationToken`[#](#signal-activation-token)

| `**org.freedesktop.Notifications.ActivationToken** (` | UINT32 `id`, |
| --- | --- |
|  | STRING `activation_token``)`; |

 This signal can be emitted before a `ActionInvoked` signal. It carries an activation token that can be used to activate a toplevel.

###### Table 10: ActivationToken Parameters [#](#id-1.10.4.4.4)

| Name | Type | Description |
| --- | --- | --- |
| *id* | UINT32 | The ID of the notification emitting the `ActionInvoked` signal. |
| *activation_token* | STRING | An activation token. This can be either an X11-style startup ID (see [Startup notification protocol (https://specifications.freedesktop.org/startup-notification-spec/startup-notification-latest.txt)](https://specifications.freedesktop.org/startup-notification-spec/startup-notification-latest.txt)) or a [Wayland xdg-activation (https://gitlab.freedesktop.org/wayland/wayland-protocols/-/tree/main/staging/xdg-activation)](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/tree/main/staging/xdg-activation) token. |

###### Note

 Clients should not assume the server will generate this signal. Some servers may not support user interaction at all, or may not support the concept of being able to generate an activation token for a notification.

© 2026 Freedesktop.org
