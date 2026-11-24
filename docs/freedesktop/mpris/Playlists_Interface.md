# Interface MediaPlayer2.Playlists

[Interface Index](https://specifications.freedesktop.org/mpris/latest/index.html) ([Compact](https://specifications.freedesktop.org/mpris/latest/interfaces.html)) | [Summary](#summary) | [Description](#description) | [Methods](#methods) | [Signals](#signals) | [Properties](#properties) | [Types](#types)

### Methods

| [ActivatePlaylist](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Method:ActivatePlaylist) | (o: PlaylistId) | → | nothing |  |
| --- | --- | --- | --- | --- |
| [GetPlaylists](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Method:GetPlaylists) | (u: Index, u: MaxCount, s: Order, b: ReverseOrder) | → | a(oss): Playlists |  |

### Signals

| [PlaylistChanged](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Signal:PlaylistChanged) | ((oss): Playlist) |  |
| --- | --- | --- |

### Properties

| [PlaylistCount](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Property:PlaylistCount) | u | Read only |  |  |
| --- | --- | --- | --- | --- |
| [Orderings](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Property:Orderings) | as ([ Playlist_Ordering_List](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Enum:Playlist_Ordering)) | Read only |  |  |
| [ActivePlaylist](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Property:ActivePlaylist) | (b(oss)) ([ Maybe_Playlist](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Struct:Maybe_Playlist)) | Read only |  |  |

### Types

| [Playlist_Id](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Simple-Type:Playlist_Id) | Simple Type | o |  |
| --- | --- | --- | --- |
| [Uri](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Simple-Type:Uri) | Simple Type | s |  |
| [Playlist_Ordering](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Enum:Playlist_Ordering) | Enum | s |  |
| [Playlist](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Struct:Playlist) | Struct | (oss) |  |
| [Maybe_Playlist](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Struct:Maybe_Playlist) | Struct | (b(oss)) |  |

Added in 2.1.  Objects implementing this interface must also implement:

### Description

Provides access to the media player's playlists.

 Since D-Bus does not provide an easy way to check for what interfaces are exported on an object, clients should attempt to get one of the properties on this interface to see if it is implemented.

# Methods

([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Method:ActivatePlaylist))

## ActivatePlaylist (o: PlaylistId) → nothing

### Parameters

-  PlaylistId — o

The id of the playlist to activate.

 Starts playing the given playlist.

 Note that this must be implemented. If the media player does not allow clients to change the playlist, it should not implement this interface at all.

 It is up to the media player whether this completely replaces the current tracklist, or whether it is merely inserted into the tracklist and the first track starts. For example, if the media player is operating in a "jukebox" mode, it may just append the playlist to the list of upcoming tracks, and skip to the first track in the playlist.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Method:GetPlaylists))

## GetPlaylists (u: Index, u: MaxCount, s: Order, b: ReverseOrder) → a(oss): Playlists

### Parameters

-  Index — u

The index of the first playlist to be fetched (according to the ordering).

-  MaxCount — u

The maximum number of playlists to fetch.

-  Order — s ([Playlist_Ordering](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Enum:Playlist_Ordering))

The ordering that should be used.

-  ReverseOrder — b

Whether the order should be reversed.

### Returns

-  Playlists — a(oss) ([Playlist_List](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Struct:Playlist))

A list of (at most MaxCount) playlists.

Gets a set of playlists.

# Signals

([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Signal:PlaylistChanged))

## PlaylistChanged ((oss): Playlist)

### Parameters

-  Playlist — (oss) ([Playlist](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Struct:Playlist))
 The playlist which details have changed.

Indicates that either the Name or Icon attribute of a playlist has changed.

Client implementations should be aware that this signal may not be implemented.

##### Rationale:

 Without this signal, media players have no way to notify clients of a change in the attributes of a playlist other than the active one

# Properties

 Accessed using the [org.freedesktop.DBus.Properties](http://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-properties) interface. ([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Property:PlaylistCount))

##  PlaylistCount — u

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 The number of playlists available.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Property:Orderings))

##  Orderings — as ([Playlist_Ordering_List](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Enum:Playlist_Ordering))

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 The available orderings. At least one must be offered.

##### Rationale:

 Media players may not have access to all the data required for some orderings. For example, creation times are not available on UNIX filesystems (don't let the ctime fool you!). On the other hand, clients should have some way to get the "most recent" playlists.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Property:ActivePlaylist))

##  ActivePlaylist — (b(oss)) ([Maybe_Playlist](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Struct:Maybe_Playlist))

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 The currently-active playlist.

 If there is no currently-active playlist, the structure's Valid field will be false, and the Playlist details are undefined.

 Note that this may not have a value even after ActivatePlaylist is called with a valid playlist id as ActivatePlaylist implementations have the option of simply inserting the contents of the playlist into the current tracklist.

# Types

Simple Type ([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Simple-Type:Playlist_Id))

##  Playlist_Id — o

Unique playlist identifier.

##### Rationale:

 Multiple playlists may have the same name.

 This is a D-Bus object id as that is the definitive way to have unique identifiers on D-Bus. It also allows for future optional expansions to the specification where tracks are exported to D-Bus with an interface similar to org.gnome.UPnP.MediaItem2.

Simple Type ([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Simple-Type:Uri))

##  Uri — s

A URI.

Enum ([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Enum:Playlist_Ordering))

##  Playlist_Ordering — s

Specifies the ordering of returned playlists.

- Alphabetical (Alphabetical)

Alphabetical ordering by name, ascending.

- CreationDate (Created)

Ordering by creation date, oldest first.

- ModifiedDate (Modified)

Ordering by last modified date, oldest first.

- LastPlayDate (Played)

Ordering by date of last playback, oldest first.

- UserDefined (User)

A user-defined ordering.

##### Rationale:

 Some media players may allow users to order playlists as they wish. This ordering allows playlists to be retreived in that order.

Struct ([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Struct:Playlist))

##  Playlist — (oss)

A data structure describing a playlist.

- Id — o ([Playlist_Id](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Simple-Type:Playlist_Id))

A unique identifier for the playlist.

This should remain the same if the playlist is renamed.

- Name — s

The name of the playlist, typically given by the user.

- Icon — s ([Uri](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Simple-Type:Uri))

The URI of an (optional) icon.

Struct ([Permalink](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Struct:Maybe_Playlist))

##  Maybe_Playlist — (b(oss))

A data structure describing a playlist, or nothing.

##### Rationale:

 D-Bus does not (at the time of writing) support a MAYBE type, so we are forced to invent our own.

- Valid — b

Whether this structure refers to a valid playlist.

- Playlist — (oss) ([Playlist](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Struct:Playlist))

The playlist, providing Valid is true, otherwise undefined.

 When constructing this type, it should be noted that the playlist ID must be a valid object path, or D-Bus implementations may reject it. This is true even when Valid is false. It is suggested that "/" is used as the playlist ID in this case.
