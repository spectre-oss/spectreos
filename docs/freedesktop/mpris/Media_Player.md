# Interface MediaPlayer2

[Interface Index](https://specifications.freedesktop.org/mpris/latest/index.html) ([Compact](https://specifications.freedesktop.org/mpris/latest/interfaces.html)) | [Summary](#summary) | [Methods](#methods) | [Properties](#properties)

### Methods

| [Raise](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Method:Raise) | () | → | nothing |  |
| --- | --- | --- | --- | --- |
| [Quit](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Method:Quit) | () | → | nothing |  |

### Properties

| [CanQuit](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanQuit) | b | Read only |  |  |
| --- | --- | --- | --- | --- |
| [Fullscreen](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:Fullscreen) | b | Read/Write |  |  |
| [CanSetFullscreen](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanSetFullscreen) | b | Read only |  |  |
| [CanRaise](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanRaise) | b | Read only |  |  |
| [HasTrackList](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:HasTrackList) | b | Read only |  |  |
| [Identity](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:Identity) | s | Read only |  |  |
| [DesktopEntry](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:DesktopEntry) | s | Read only |  |  |
| [SupportedUriSchemes](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:SupportedUriSchemes) | as | Read only |  |  |
| [SupportedMimeTypes](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:SupportedMimeTypes) | as | Read only |  |  |

 Objects implementing this interface must also implement:

# Methods

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Method:Raise))

## Raise () → nothing

 Brings the media player's user interface to the front using any appropriate mechanism available.

 The media player may be unable to control how its user interface is displayed, or it may not have a graphical user interface at all. In this case, the [CanRaise](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanRaise) property is **false** and this method does nothing.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Method:Quit))

## Quit () → nothing

Causes the media player to stop running.

 The media player may refuse to allow clients to shut it down. In this case, the [CanQuit](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanQuit) property is **false** and this method does nothing.

 Note: Media players which can be D-Bus activated, or for which there is no sensibly easy way to terminate a running instance (via the main interface or a notification area icon for example) should allow clients to use this method. Otherwise, it should not be needed.

If the media player does not have a UI, this should be implemented.

# Properties

 Accessed using the [org.freedesktop.DBus.Properties](http://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-properties) interface. ([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanQuit))

##  CanQuit — b

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 If **false**, calling [Quit](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Method:Quit) will have no effect, and may raise a NotSupported error. If **true**, calling [Quit](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Method:Quit) will cause the media application to attempt to quit (although it may still be prevented from quitting by the user, for example).

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:Fullscreen))

##  Fullscreen — b

Read/Write Added in 2.2.  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

Whether the media player is occupying the fullscreen.

 This is typically used for videos. A value of **true** indicates that the media player is taking up the full screen.

 Media centre software may well have this value fixed to **true**

 If [CanSetFullscreen](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanSetFullscreen) is **true**, clients may set this property to **true** to tell the media player to enter fullscreen mode, or to **false** to return to windowed mode.

 If [CanSetFullscreen](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanSetFullscreen) is **false**, then attempting to set this property should have no effect, and may raise an error. However, even if it is **true**, the media player may still be unable to fulfil the request, in which case attempting to set this property will have no effect (but should not raise an error).

##### Rationale:

 This allows remote control interfaces, such as LIRC or mobile devices like phones, to control whether a video is shown in fullscreen.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanSetFullscreen))

##  CanSetFullscreen — b

Read only Added in 2.2.  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 If **false**, attempting to set [Fullscreen](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:Fullscreen) will have no effect, and may raise an error. If **true**, attempting to set [Fullscreen](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:Fullscreen) will not raise an error, and (if it is different from the current value) will cause the media player to attempt to enter or exit fullscreen mode.

 Note that the media player may be unable to fulfil the request. In this case, the value will not change. If the media player knows in advance that it will not be able to fulfil the request, however, this property should be **false**.

##### Rationale:

 This allows clients to choose whether to display controls for entering or exiting fullscreen mode.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:CanRaise))

##  CanRaise — b

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 If **false**, calling [Raise](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Method:Raise) will have no effect, and may raise a NotSupported error. If **true**, calling [Raise](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Method:Raise) will cause the media application to attempt to bring its user interface to the front, although it may be prevented from doing so (by the window manager, for example).

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:HasTrackList))

##  HasTrackList — b

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 Indicates whether the **/org/mpris/MediaPlayer2** object implements the **org.mpris.MediaPlayer2.TrackList** interface.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:Identity))

##  Identity — s

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

A friendly name to identify the media player to users.

This should usually match the name found in .desktop files

(eg: "VLC media player").

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:DesktopEntry))

##  DesktopEntry — s

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

The basename of an installed .desktop file which complies with the [Desktop entry specification](http://standards.freedesktop.org/desktop-entry-spec/latest/), with the ".desktop" extension stripped.

 Example: The desktop entry file is "/usr/share/applications/vlc.desktop", and this property contains "vlc"

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:SupportedUriSchemes))

##  SupportedUriSchemes — as

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 The URI schemes supported by the media player.

 This can be viewed as protocols supported by the player in almost all cases. Almost every media player will include support for the "file" scheme. Other common schemes are "http" and "rtsp".

 Note that URI schemes should be lower-case.

##### Rationale:

 This is important for clients to know when using the editing capabilities of the Playlist interface, for example.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Media_Player.html#Property:SupportedMimeTypes))

##  SupportedMimeTypes — as

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 The mime-types supported by the media player.

 Mime-types should be in the standard format (eg: audio/mpeg or application/ogg).

##### Rationale:

 This is important for clients to know when using the editing capabilities of the Playlist interface, for example.
