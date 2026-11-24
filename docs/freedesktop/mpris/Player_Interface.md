# Interface MediaPlayer2.Player

[Interface Index](https://specifications.freedesktop.org/mpris/latest/index.html) ([Compact](https://specifications.freedesktop.org/mpris/latest/interfaces.html)) | [Summary](#summary) | [Description](#description) | [Methods](#methods) | [Signals](#signals) | [Properties](#properties) | [Types](#types)

### Methods

| [Next](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Next) | () | → | nothing |  |
| --- | --- | --- | --- | --- |
| [Previous](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Previous) | () | → | nothing |  |
| [Pause](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Pause) | () | → | nothing |  |
| [PlayPause](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:PlayPause) | () | → | nothing |  |
| [Stop](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Stop) | () | → | nothing |  |
| [Play](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Play) | () | → | nothing |  |
| [Seek](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Seek) | (x: Offset) | → | nothing |  |
| [SetPosition](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:SetPosition) | (o: TrackId, x: Position) | → | nothing |  |
| [OpenUri](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:OpenUri) | (s: Uri) | → | nothing |  |

### Signals

| [Seeked](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Signal:Seeked) | (x: Position) |  |
| --- | --- | --- |

### Properties

| [PlaybackStatus](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:PlaybackStatus) | s ([ Playback_Status](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Enum:Playback_Status)) | Read only |  |  |
| --- | --- | --- | --- | --- |
| [LoopStatus](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:LoopStatus) | s ([ Loop_Status](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Enum:Loop_Status)) | Read/Write |  |  |
| [Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Rate) | d ([ Playback_Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Playback_Rate)) | Read/Write |  |  |
| [Shuffle](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Shuffle) | b | Read/Write |  |  |
| [Metadata](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Metadata) | a{sv} ([ Metadata_Map](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Mapping:Metadata_Map)) | Read only |  |  |
| [Volume](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Volume) | d ([ Volume](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Volume)) | Read/Write |  |  |
| [Position](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Position) | x ([ Time_In_Us](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Time_In_Us)) | Read only |  |  |
| [MinimumRate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:MinimumRate) | d ([ Playback_Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Playback_Rate)) | Read only |  |  |
| [MaximumRate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:MaximumRate) | d ([ Playback_Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Playback_Rate)) | Read only |  |  |
| [CanGoNext](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanGoNext) | b | Read only |  |  |
| [CanGoPrevious](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanGoPrevious) | b | Read only |  |  |
| [CanPlay](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanPlay) | b | Read only |  |  |
| [CanPause](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanPause) | b | Read only |  |  |
| [CanSeek](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanSeek) | b | Read only |  |  |
| [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) | b | Read only |  |  |

### Types

| [Track_Id](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id) | Simple Type | o |  |
| --- | --- | --- | --- |
| [Playback_Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Playback_Rate) | Simple Type | d |  |
| [Volume](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Volume) | Simple Type | d |  |
| [Time_In_Us](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Time_In_Us) | Simple Type | x |  |
| [Playback_Status](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Enum:Playback_Status) | Enum | s |  |
| [Loop_Status](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Enum:Loop_Status) | Enum | s |  |

 Objects implementing this interface must also implement:

### Description

 This interface implements the methods for querying and providing basic control over what is currently playing.

# Methods

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Next))

## Next () → nothing

Skips to the next track in the tracklist.

 If there is no next track (and endless playback and track repeat are both off), stop playback.

If playback is paused or stopped, it remains that way.

 If [CanGoNext](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanGoNext) is **false**, attempting to call this method should have no effect.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Previous))

## Previous () → nothing

Skips to the previous track in the tracklist.

 If there is no previous track (and endless playback and track repeat are both off), stop playback.

If playback is paused or stopped, it remains that way.

 If [CanGoPrevious](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanGoPrevious) is **false**, attempting to call this method should have no effect.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Pause))

## Pause () → nothing

Pauses playback.

If playback is already paused, this has no effect.

 Calling Play after this should cause playback to start again from the same position.

 If [CanPause](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanPause) is **false**, attempting to call this method should have no effect.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:PlayPause))

## PlayPause () → nothing

Pauses playback.

If playback is already paused, resumes playback.

If playback is stopped, starts playback.

 If [CanPause](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanPause) is **false**, attempting to call this method should have no effect and raise an error.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Stop))

## Stop () → nothing

Stops playback.

If playback is already stopped, this has no effect.

 Calling Play after this should cause playback to start again from the beginning of the track.

 If [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **false**, attempting to call this method should have no effect and raise an error.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Play))

## Play () → nothing

Starts or resumes playback.

If already playing, this has no effect.

If paused, playback resumes from the current position.

If there is no track to play, this has no effect.

 If [CanPlay](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanPlay) is **false**, attempting to call this method should have no effect.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Seek))

## Seek (x: Offset) → nothing

### Parameters

-  Offset — x ([Time_In_Us](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Time_In_Us))

The number of microseconds to seek forward.

 Seeks forward in the current track by the specified number of microseconds.

 A negative value seeks back. If this would mean seeking back further than the start of the track, the position is set to 0.

 If the value passed in would mean seeking beyond the end of the track, acts like a call to Next.

 If the [CanSeek](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanSeek) property is false, this has no effect.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:SetPosition))

## SetPosition (o: TrackId, x: Position) → nothing

### Parameters

-  TrackId — o ([Track_Id](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

The currently playing track's identifier.

 If this does not match the id of the currently-playing track, the call is ignored as "stale".

/org/mpris/MediaPlayer2/TrackList/NoTrack is *not* a valid value for this argument.

-  Position — x ([Time_In_Us](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Time_In_Us))

Track position in microseconds.

This must be between 0 and <track_length>.

Sets the current track position in microseconds.

If the Position argument is less than 0, do nothing.

 If the Position argument is greater than the track length, do nothing.

 If the [CanSeek](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanSeek) property is false, this has no effect.

##### Rationale:

 The reason for having this method, rather than making [Position](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Position) writable, is to include the TrackId argument to avoid race conditions where a client tries to seek to a position when the track has already changed.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:OpenUri))

## OpenUri (s: Uri) → nothing

### Parameters

-  Uri — s ([Uri](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Simple-Type:Uri))

 Uri of the track to load. Its uri scheme should be an element of the org.mpris.MediaPlayer2.SupportedUriSchemes property and the mime-type should match one of the elements of the org.mpris.MediaPlayer2.SupportedMimeTypes.

Opens the Uri given as an argument

If the playback is stopped, starts playing

 If the uri scheme or the mime-type of the uri to open is not supported, this method does nothing and may raise an error. In particular, if the list of available uri schemes is empty, this method may not be implemented.

Clients should not assume that the Uri has been opened as soon as this method returns. They should wait until the mpris:trackid field in the [Metadata](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Metadata) property changes.

 If the media player implements the TrackList interface, then the opened track should be made part of the tracklist, the org.mpris.MediaPlayer2.TrackList.TrackAdded or org.mpris.MediaPlayer2.TrackList.TrackListReplaced signal should be fired, as well as the org.freedesktop.DBus.Properties.PropertiesChanged signal on the tracklist interface.

# Signals

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Signal:Seeked))

## Seeked (x: Position)

### Parameters

-  Position — x ([Time_In_Us](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Time_In_Us))

The new position, in microseconds.

 Indicates that the track position has changed in a way that is inconsistant with the current playing state.

When this signal is not received, clients should assume that:

-  When playing, the position progresses according to the rate property.
- When paused, it remains constant.

 This signal does not need to be emitted when playback starts or when the track changes, unless the track is starting at an unexpected position. An expected position would be the last known one when going from Paused to Playing, and 0 when going from Stopped to Playing.

# Properties

 Accessed using the [org.freedesktop.DBus.Properties](http://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-properties) interface. ([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:PlaybackStatus))

##  PlaybackStatus — s ([Playback_Status](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Enum:Playback_Status))

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

The current playback status.

 May be "Playing", "Paused" or "Stopped".

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:LoopStatus))

##  LoopStatus — s ([Loop_Status](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Enum:Loop_Status))

Read/Write  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

The current loop / repeat status

May be:
- "None" if the playback will stop when there are no more tracks to play
- "Track" if the current track will start again from the begining once it has finished playing
- "Playlist" if the playback loops through a list of tracks

 If [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **false**, attempting to set this property should have no effect and raise an error.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Rate))

##  Rate — d ([Playback_Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Playback_Rate))

Read/Write  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

The current playback rate.

 The value must fall in the range described by [MinimumRate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:MinimumRate) and [MaximumRate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:MaximumRate), and must not be 0.0. If playback is paused, the [PlaybackStatus](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:PlaybackStatus) property should be used to indicate this. A value of 0.0 should not be set by the client. If it is, the media player should act as though [Pause](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Pause) was called.

 If the media player has no ability to play at speeds other than the normal playback rate, this must still be implemented, and must return 1.0. The [MinimumRate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:MinimumRate) and [MaximumRate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:MaximumRate) properties must also be set to 1.0.

 Not all values may be accepted by the media player. It is left to media player implementations to decide how to deal with values they cannot use; they may either ignore them or pick a "best fit" value. Clients are recommended to only use sensible fractions or multiples of 1 (eg: 0.5, 0.25, 1.5, 2.0, etc).

##### Rationale:

 This allows clients to display (reasonably) accurate progress bars without having to regularly query the media player for the current position.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Shuffle))

##  Shuffle — b

Read/Write  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 A value of **false** indicates that playback is progressing linearly through a playlist, while **true** means playback is progressing through a playlist in some other order.

 If [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **false**, attempting to set this property should have no effect and raise an error.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Metadata))

##  Metadata — a{sv} ([Metadata_Map](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Mapping:Metadata_Map))

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

The metadata of the current element.

 If there is a current track, this must have a "mpris:trackid" entry (of D-Bus type "o") at the very least, which contains a D-Bus path that uniquely identifies this track.

 See the type documentation for more details.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Volume))

##  Volume — d ([Volume](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Volume))

Read/Write  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

The volume level.

 When setting, if a negative value is passed, the volume should be set to 0.0.

 If [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **false**, attempting to set this property should have no effect and raise an error.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Position))

##  Position — x ([Time_In_Us](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Time_In_Us))

Read only  The `org.freedesktop.DBus.Properties.PropertiesChanged` signal is **not** emitted when this property changes.

 The current track position in microseconds, between 0 and the 'mpris:length' metadata entry (see Metadata).

 Note: If the media player allows it, the current playback position can be changed either the SetPosition method or the Seek method on this interface. If this is not the case, the [CanSeek](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanSeek) property is false, and setting this property has no effect and can raise an error.

 If the playback progresses in a way that is inconstistant with the [Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Rate) property, the [Seeked](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Signal:Seeked) signal is emited.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:MinimumRate))

##  MinimumRate — d ([Playback_Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Playback_Rate))

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 The minimum value which the [Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Rate) property can take. Clients should not attempt to set the [Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Rate) property below this value.

 Note that even if this value is 0.0 or negative, clients should not attempt to set the [Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Rate) property to 0.0.

This value should always be 1.0 or less.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:MaximumRate))

##  MaximumRate — d ([Playback_Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Playback_Rate))

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 The maximum value which the [Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Rate) property can take. Clients should not attempt to set the [Rate](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:Rate) property above this value.

 This value should always be 1.0 or greater.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanGoNext))

##  CanGoNext — b

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 Whether the client can call the [Next](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Next) method on this interface and expect the current track to change.

 If it is unknown whether a call to [Next](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Next) will be successful (for example, when streaming tracks), this property should be set to **true**.

 If [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **false**, this property should also be **false**.

##### Rationale:

 Even when playback can generally be controlled, there may not always be a next track to move to.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanGoPrevious))

##  CanGoPrevious — b

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 Whether the client can call the [Previous](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Previous) method on this interface and expect the current track to change.

 If it is unknown whether a call to [Previous](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Previous) will be successful (for example, when streaming tracks), this property should be set to **true**.

 If [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **false**, this property should also be **false**.

##### Rationale:

 Even when playback can generally be controlled, there may not always be a next previous to move to.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanPlay))

##  CanPlay — b

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

Whether playback can be started using [Play](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Play) or [PlayPause](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:PlayPause).

 Note that this is related to whether there is a "current track": the value should not depend on whether the track is currently paused or playing. In fact, if a track is currently playing (and [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **true**), this should be **true**.

 If [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **false**, this property should also be **false**.

##### Rationale:

 Even when playback can generally be controlled, it may not be possible to enter a "playing" state, for example if there is no "current track".

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanPause))

##  CanPause — b

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

Whether playback can be paused using [Pause](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Pause) or [PlayPause](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:PlayPause).

 Note that this is an intrinsic property of the current track: its value should not depend on whether the track is currently paused or playing. In fact, if playback is currently paused (and [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **true**), this should be **true**.

 If [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **false**, this property should also be **false**.

##### Rationale:

 Not all media is pausable: it may not be possible to pause some streamed media, for example.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanSeek))

##  CanSeek — b

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 Whether the client can control the playback position using [Seek](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:Seek) and [SetPosition](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Method:SetPosition). This may be different for different tracks.

 If [CanControl](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl) is **false**, this property should also be **false**.

##### Rationale:

 Not all media is seekable: it may not be possible to seek when playing some streamed media, for example.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Property:CanControl))

##  CanControl — b

Read only  The `org.freedesktop.DBus.Properties.PropertiesChanged` signal is **not** emitted when this property changes.

Whether the media player may be controlled over this interface.

 This property is not expected to change, as it describes an intrinsic capability of the implementation.

 If this is **false**, clients should assume that all properties on this interface are read-only (and will raise errors if writing to them is attempted), no methods are implemented and all other properties starting with "Can" are also **false**.

##### Rationale:

 This allows clients to determine whether to present and enable controls to the user in advance of attempting to call methods and write to properties.

# Types

Simple Type ([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

##  Track_Id — o

Unique track identifier.

 If the media player implements the TrackList interface and allows the same track to appear multiple times in the tracklist, this must be unique within the scope of the tracklist.

 Note that this should be a valid D-Bus object id, although clients should not assume that any object is actually exported with any interfaces at that path.

 Media players may not use any paths starting with /org/mpris unless explicitly allowed by this specification. Such paths are intended to have special meaning, such as /org/mpris/MediaPlayer2/TrackList/NoTrack to indicate "no track".

##### Rationale:

 This is a D-Bus object id as that is the definitive way to have unique identifiers on D-Bus. It also allows for future optional expansions to the specification where tracks are exported to D-Bus with an interface similar to org.gnome.UPnP.MediaItem2.

Simple Type ([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Playback_Rate))

##  Playback_Rate — d

A playback rate

 This is a multiplier, so a value of 0.5 indicates that playback is happening at half speed, while 1.5 means that 1.5 seconds of "track time" is consumed every second.

Simple Type ([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Volume))

##  Volume — d

Audio volume level

- 0.0 means mute.
- 1.0 is a sensible maximum volume level (ex: 0dB).

 Note that the volume may be higher than 1.0, although generally clients should not attempt to set it above 1.0.

Simple Type ([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Time_In_Us))

##  Time_In_Us — x

Time in microseconds.

Enum ([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Enum:Playback_Status))

##  Playback_Status — s

A playback state.

- Playing (Playing)

A track is currently playing.

- Paused (Paused)

A track is currently paused.

- Stopped (Stopped)

There is no track currently playing.

Enum ([Permalink](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Enum:Loop_Status))

##  Loop_Status — s

A repeat / loop status

- None (None)

The playback will stop when there are no more tracks to play

- Track (Track)

The current track will start again from the begining once it has finished playing

- Playlist (Playlist)

The playback loops through a list of tracks
