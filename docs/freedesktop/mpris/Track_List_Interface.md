# Interface MediaPlayer2.TrackList

[Interface Index](https://specifications.freedesktop.org/mpris/latest/index.html) ([Compact](https://specifications.freedesktop.org/mpris/latest/interfaces.html)) | [Summary](#summary) | [Description](#description) | [Methods](#methods) | [Signals](#signals) | [Properties](#properties) | [Types](#types)

### Methods

| [GetTracksMetadata](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:GetTracksMetadata) | (ao: TrackIds) | → | aa{sv}: Metadata |  |
| --- | --- | --- | --- | --- |
| [AddTrack](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:AddTrack) | (s: Uri, o: AfterTrack, b: SetAsCurrent) | → | nothing |  |
| [RemoveTrack](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:RemoveTrack) | (o: TrackId) | → | nothing |  |
| [GoTo](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:GoTo) | (o: TrackId) | → | nothing |  |

### Signals

| [TrackListReplaced](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackListReplaced) | (ao: Tracks, o: CurrentTrack) |  |
| --- | --- | --- |
| [TrackAdded](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackAdded) | (a{sv}: Metadata, o: AfterTrack) |  |
| [TrackRemoved](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackRemoved) | (o: TrackId) |  |
| [TrackMetadataChanged](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackMetadataChanged) | (o: TrackId, a{sv}: Metadata) |  |

### Properties

| [Tracks](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Property:Tracks) | ao ([ Track_Id_List](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id)) | Read only |  |  |
| --- | --- | --- | --- | --- |
| [CanEditTracks](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Property:CanEditTracks) | b | Read only |  |  |

### Types

| [Uri](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Simple-Type:Uri) | Simple Type | s |  |
| --- | --- | --- | --- |
| [Metadata_Map](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Mapping:Metadata_Map) | Mapping | a{sv} |  |

 Objects implementing this interface must also implement:

### Description

 Provides access to a short list of tracks which were recently played or will be played shortly. This is intended to provide context to the currently-playing track, rather than giving complete access to the media player's playlist.

 Example use cases are the list of tracks from the same album as the currently playing song or the [Rhythmbox](http://projects.gnome.org/rhythmbox/) play queue.

 Each track in the tracklist has a unique identifier. The intention is that this uniquely identifies the track within the scope of the tracklist. In particular, if a media item (a particular music file, say) occurs twice in the track list, each occurrence should have a different identifier. If a track is removed from the middle of the playlist, it should not affect the track ids of any other tracks in the tracklist.

 As a result, the traditional track identifiers of URLs and position in the playlist cannot be used. Any scheme which satisfies the uniqueness requirements is valid, as clients should not make any assumptions about the value of the track id beyond the fact that it is a unique identifier.

 Note that the (memory and processing) burden of implementing the TrackList interface and maintaining unique track ids for the playlist can be mitigated by only exposing a subset of the playlist when it is very long (the 20 or so tracks around the currently playing track, for example). This is a recommended practice as the tracklist interface is not designed to enable browsing through a large list of tracks, but rather to provide clients with context about the currently playing track.

# Methods

([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:GetTracksMetadata))

## GetTracksMetadata (ao: TrackIds) → aa{sv}: Metadata

### Parameters

-  TrackIds — ao ([Track_Id_List](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

The list of track ids for which metadata is requested.

### Returns

-  Metadata — aa{sv} ([Metadata_Map_List](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Mapping:Metadata_Map))

Metadata of the set of tracks given as input.

See the type documentation for more details.

Gets all the metadata available for a set of tracks.

 Each set of metadata must have a "mpris:trackid" entry at the very least, which contains a string that uniquely identifies this track within the scope of the tracklist.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:AddTrack))

## AddTrack (s: Uri, o: AfterTrack, b: SetAsCurrent) → nothing

### Parameters

-  Uri — s ([Uri](https://specifications.freedesktop.org/mpris/latest/Playlists_Interface.html#Simple-Type:Uri))

 The uri of the item to add. Its uri scheme should be an element of the **org.mpris.MediaPlayer2.SupportedUriSchemes** property and the mime-type should match one of the elements of the **org.mpris.MediaPlayer2.SupportedMimeTypes**

-  AfterTrack — o ([Track_Id](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

 The identifier of the track after which the new item should be inserted. The path /org/mpris/MediaPlayer2/TrackList/NoTrack indicates that the track should be inserted at the start of the track list.

-  SetAsCurrent — b

 Whether the newly inserted track should be considered as the current track. Setting this to true has the same effect as calling GoTo afterwards.

Adds a URI in the TrackList.

 If the [CanEditTracks](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Property:CanEditTracks) property is false, this has no effect.

 Note: Clients should not assume that the track has been added at the time when this method returns. They should wait for a TrackAdded (or TrackListReplaced) signal.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:RemoveTrack))

## RemoveTrack (o: TrackId) → nothing

### Parameters

-  TrackId — o ([Track_Id](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

Identifier of the track to be removed.

/org/mpris/MediaPlayer2/TrackList/NoTrack is *not* a valid value for this argument.

Removes an item from the TrackList.

If the track is not part of this tracklist, this has no effect.

 If the [CanEditTracks](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Property:CanEditTracks) property is false, this has no effect.

 Note: Clients should not assume that the track has been removed at the time when this method returns. They should wait for a TrackRemoved (or TrackListReplaced) signal.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:GoTo))

## GoTo (o: TrackId) → nothing

### Parameters

-  TrackId — o ([Track_Id](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

Identifier of the track to skip to.

/org/mpris/MediaPlayer2/TrackList/NoTrack is *not* a valid value for this argument.

Skip to the specified TrackId.

If the track is not part of this tracklist, this has no effect.

 If this object is not **/org/mpris/MediaPlayer2**, the current TrackList's tracks should be replaced with the contents of this TrackList, and the TrackListReplaced signal should be fired from **/org/mpris/MediaPlayer2**.

# Signals

([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackListReplaced))

## TrackListReplaced (ao: Tracks, o: CurrentTrack)

### Parameters

-  Tracks — ao ([Track_Id_List](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

The new content of the tracklist.

-  CurrentTrack — o ([Track_Id](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

The identifier of the track to be considered as current.

/org/mpris/MediaPlayer2/TrackList/NoTrack indicates that there is no current track.

 This should correspond to the mpris:trackid field of the Metadata property of the org.mpris.MediaPlayer2.Player interface.

Indicates that the entire tracklist has been replaced.

 It is left up to the implementation to decide when a change to the track list is invasive enough that this signal should be emitted instead of a series of TrackAdded and TrackRemoved signals.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackAdded))

## TrackAdded (a{sv}: Metadata, o: AfterTrack)

### Parameters

-  Metadata — a{sv} ([Metadata_Map](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Mapping:Metadata_Map))

The metadata of the newly added item.

This must include a mpris:trackid entry.

See the type documentation for more details.

-  AfterTrack — o ([Track_Id](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

 The identifier of the track after which the new track was inserted. The path /org/mpris/MediaPlayer2/TrackList/NoTrack indicates that the track was inserted at the start of the track list.

Indicates that a track has been added to the track list.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackRemoved))

## TrackRemoved (o: TrackId)

### Parameters

-  TrackId — o ([Track_Id](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

The identifier of the track being removed.

/org/mpris/MediaPlayer2/TrackList/NoTrack is *not* a valid value for this argument.

Indicates that a track has been removed from the track list.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackMetadataChanged))

## TrackMetadataChanged (o: TrackId, a{sv}: Metadata)

### Parameters

-  TrackId — o ([Track_Id](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

The id of the track which metadata has changed.

If the track id has changed, this will be the old value.

/org/mpris/MediaPlayer2/TrackList/NoTrack is *not* a valid value for this argument.

-  Metadata — a{sv} ([Metadata_Map](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Mapping:Metadata_Map))

The new track metadata.

 This must include a mpris:trackid entry. If the track id has changed, this will be the new value.

See the type documentation for more details.

 Indicates that the metadata of a track in the tracklist has changed.

 This may indicate that a track has been replaced, in which case the mpris:trackid metadata entry is different from the TrackId argument.

# Properties

 Accessed using the [org.freedesktop.DBus.Properties](http://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-properties) interface. ([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Property:Tracks))

##  Tracks — ao ([Track_Id_List](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html#Simple-Type:Track_Id))

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted, but the new value is not sent.

 An array which contains the identifier of each track in the tracklist, in order.

 The org.freedesktop.DBus.Properties.PropertiesChanged signal is emited every time this property changes, but the signal message does not contain the new value. Client implementations should rather rely on the [TrackAdded](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackAdded), [TrackRemoved](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackRemoved) and [TrackListReplaced](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Signal:TrackListReplaced) signals to keep their representation of the tracklist up to date.

([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Property:CanEditTracks))

##  CanEditTracks — b

Read only  When this property changes, the `org.freedesktop.DBus.Properties.PropertiesChanged` signal is emitted with the new value.

 If **false**, calling [AddTrack](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:AddTrack) or [RemoveTrack](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Method:RemoveTrack) will have no effect, and may raise a NotSupported error.

# Types

Simple Type ([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Simple-Type:Uri))

##  Uri — s

A unique resource identifier.

Mapping ([Permalink](https://specifications.freedesktop.org/mpris/latest/Track_List_Interface.html#Mapping:Metadata_Map))

##  Metadata_Map — a{sv}

A mapping from metadata attribute names to values.

 The **mpris:trackid** attribute must always be present, and must be of D-Bus type "o". This contains a D-Bus path that uniquely identifies the track within the scope of the playlist. There may or may not be an actual D-Bus object at that path; this specification says nothing about what interfaces such an object may implement.

 If the length of the track is known, it should be provided in the metadata property with the "mpris:length" key. The length must be given in microseconds, and be represented as a signed 64-bit integer.

 If there is an image associated with the track, a URL for it may be provided using the "mpris:artUrl" key. For other metadata, fields defined by the Xesam ontology should be used, prefixed by "xesam:". See the [metadata page on the freedesktop.org wiki](http://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata) for a list of common fields.

 Lists of strings should be passed using the array-of-string ("as") D-Bus type. Dates should be passed as strings using the ISO 8601 extended format (eg: 2007-04-29T14:35:51). If the timezone is known, RFC 3339's internet profile should be used (eg: 2007-04-29T14:35:51+02:00).

- Attribute — s

 The name of the attribute; see the [metadata page](http://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata) for guidelines on names to use.

- Value — v

The value of the attribute, in the most appropriate format.
