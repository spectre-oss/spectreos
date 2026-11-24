[Jump to content](#_content)[Sound Theme Specification](#index)[Show Contents: Sound Theme Specification](#index)[Top](#)
1. [1 Overview](#overview)
2. [2 Definitions](#definitions)
3. [3 Directory Layout](#directory_layout)
4. [4 File Formats](#file_formats)
5. [5 Sound Lookup](#sound_lookup)
6. [6 Example](#example)
7. [7 Installing Application Sounds](#install_sounds)
8. [8 Implementation Notes](#implementation_notes)
9. [9 Background](#background)
10. [A Change history](#changes)

## Sound Theme Specification [#](#index)

Authors: Marc-Andre Lureau `<marcandre.lureau@gmail.com>`Lennart Poettering `<lennart@poettering.net>`Patryk Zawadzki `<patrys@pld-linux.org>`Alexander Larsson `<alexl@redhat.com>`Frans Englich `<frans.englich@telia.com>`
Publication Date: 4 June 2008 2008, Version: Version 0.2

- **[1 Overview](#overview)**
- **[2 Definitions](#definitions)**
- **[3 Directory Layout](#directory_layout)**
- **[4 File Formats](#file_formats)**
- **[5 Sound Lookup](#sound_lookup)**
- **[6 Example](#example)**
- **[7 Installing Application Sounds](#install_sounds)**
- **[8 Implementation Notes](#implementation_notes)**
- **[9 Background](#background)**
- **[A Change history](#changes)**

## 1 Overview [#](#overview)

 A sound theme is a set of event sounds that share a common feel, or instrument set. The user can then select the sound theme that they want to use, and all applications use sounds from the theme.

 From a programmer perspective a sound theme is just a mapping. Given a set of directories to look for sounds in and a theme name it maps from sound name to a sound filename.

## 2 Definitions [#](#definitions)

- **Sound Theme**

 A sound theme is a named set of sounds. It is used to map from a sound name. Themes may inherit from other themes as a way to extend them.

- **Sound file**

 The mandatory supported sound file formats are WAV/PCM 8-48kHz, 8/16bits, and OGG/Vorbis I. WAV at 48kHz/Stereo is the recommended uncompressed format, and OGG/Vorbis I is the recommended compressed format. The sample must be normalized, in order to be mixed down nicely with a suitable average replay level.

- **Base Directory**

 Sounds and themes are searched for in a set of directories, called base directories. The themes are stored in subdirectories of the base directories.

## 3 Directory Layout [#](#directory_layout)

 Sounds and themes are looked for in a set of directories. By default, applications should look in $XDG_DATA_DIRS/sounds. Applications may further add their own sound directories to this list, and users may extend or change the list (in application/desktop specific ways). In each of these directories themes are stored as subdirectories. A theme can be spread across several base directories by having subdirectories of the same name. This way users can extend and override system themes.

 In order to have a place for third party applications to install their sounds there should always exist a theme called "freedesktop". The data for the freedesktop theme is available for download at: [*freedesktop.org sound theme specification page* (http://www.freedesktop.org/software/sound-theme/)](http://www.freedesktop.org/software/sound-theme/). Implementations are required to look in the "freedesktop" theme if a sound was not found in the current theme.

 Each theme is stored as subdirectories of the base directories. The internal name of the theme is the name of the subdirectory, although the user-visible name as specified by the theme may be different. Hence, theme names are case sensitive, and are limited to ASCII characters. Theme names may also not contain comma or space.

 In at least one of the theme directories there must be a file called index.theme that describes the theme. The first index.theme found while searching the base directories in order is used. This file describes the general attributes of the theme.

 In the theme directory are also a set of subdirectories containing sound files. The subdirectories are allowed to be several levels deep, e.g. the subdirectory "stereo/alerts" in the theme "freedesktop" would end up at $basedir/freedesktop/stereo/alerts.

 The sounds files must be one of the types: WAV/PCM 8-48kHz, 8/16 bits or OGG/Vorbis I. The extension must be ".wav", or ".oga" respectively (in lower case). It is not recommended to use ".ogg" extension, but it is also supported for legacy reasons.

 Besides the sound files, there may be an additional file with extra sound data for each file. It should have the same basename as the sound file, with the extension ".sound". e.g. if the sound file is called "system-shutdown.wav" the corresponding file would be named "system-shutdown.sound".

 Finally, a pseudo file format ".disabled" is used for disabling sounds in a theme that inherits from another theme. If the sound lookup algorithms detects a file with the suffix ".disabled" it shall immediately terminate the lookup logic and consider the sound not available. All files with ".disabled" suffix should be of length zero.

 To save disk space Vorbis I encoded sound files are recommended.

 The sound samples must be normalized with a suitable average replay level, in order to be properly mixed down. For more informations, please read [*Replay Gain Calculation from hydrogenaudio.org* (http://replaygain.hydrogenaudio.org/calculating_rg.html)](http://replaygain.hydrogenaudio.org/calculating_rg.html) and check that your volume level is reasonable.

 Sounds for different output profiles (i.e. for Stereo, Surround 4.1, Surround 5.1 and so on) may be part of a theme. All implementations are required to fallback to the "stereo" output profile as last resort.

## 4 File Formats [#](#file_formats)

 Both the sound theme description file and the sound data files are ini-style text files, as described in the desktop file specification. They don't have an encoding field. Instead, they must always be stored in UTF-8 encoding.

 The index.theme file must start with a section called *Sound Theme*, with contents according to table 1 below. All lists are comma-separated.

###### Table 1: Standard Keys [#](#id-1.5.3.2)

| Key | Description | Value Type | Required |
| --- | --- | --- | --- |
| Name | Short name of the sound theme, used in e.g. lists when selecting themes. | localestring | YES |
| Comment | Longer string describing the theme | localestring | YES |
| Inherits | The name of the theme that this theme inherits from. If a sound name is not found in the current theme, it is searched for in the inherited theme (and recursively in all the inherited themes). If no theme is specified implementations are required to add the "freedesktop" theme to the inheritance tree. An implementation may optionally add other default themes in between the last specified theme and the freedesktop theme. | strings | NO |
| Directories | List of subdirectories for this theme. For every subdirectory there must be a section in the index.theme file describing that directory. | strings | YES |
| Hidden | Whether to hide the theme in a theme selection user interface. This is used for things such as fallback-themes that are not supposed to be visible to the user. | boolean | NO |
| Example | The name of a sound that should be used as an example of how this theme sounds like. | string | NO |

 Each directory specified in the Directory key has a corresponding section with the same name as the directory. The contents of this section is listed in table 2 below.

###### Table 2: Per-Directory Keys [#](#id-1.5.4.1)

| Key | Description | Value Type | Required | Type |
| --- | --- | --- | --- | --- |
| Context | The context the sound is normally used in. This is in detail discussed in [Section 4.1, “Context”](#context). | string | NO |  |
| OutputProfile | Define the output profile for which the sounds are made for. It can be "stereo" or "5.1", or any other output configuration. | string | NO |  |

 In addition to these groups you may add extra groups to the index.theme file in order to extend it. These extensions must begin with "X-", and can be used to add desktop specific information to the theme file. Example group names would be "X-KDE Sound Theme" or "X-Gnome Sound Theme".

 The optional file filename.sound contains a group "Sound Data", with the content listed in table 3.

###### Table 3: Sound Data Keys [#](#id-1.5.6.1)

| Key | Description | Value Type | Required |
| --- | --- | --- | --- |
| DisplayName | A translated UTF8 string that can be displayed instead of the sound by a user interface. | localestring | NO |

 Extensions to filename.sound are allowed, but the keys must be begin with "X-" to avoid collisions with future standardized extensions to this format.

### 4.1 Context [#](#context)

The `context` allows the designer to group sounds on a conceptual level. It doesn't act as a namespace in the file system, such that sounds can have identical names, but allows implementations to categorize and sort by it, for example.

These are the available contexts:

-

Alert. Sounds to alert the user of an action or event which may have a major impact on the system or their current use, such as dialog-error.

-

Notification. Sounds to notify the user that the system, or their current use has changed state in some way, e.g. new email arriving, new non-critical updates available...

-

Support. Sounds that give the user feedback on their actions. Sounds on window opening/closing for example.

-

Game. Sounds used for games.

## 5 Sound Lookup [#](#sound_lookup)

 The sound lookup mechanism has two global settings, the list of base directories and the internal name of the current theme. Given these we specify how to look up sound file from the sound name.

 The lookup is done first in the current theme, and then recursively in each of the current theme's parents, and finally in the default theme called "freedesktop" (implementations may add more default themes before "freedesktop", but "freedesktop" must be last). A last fallback is unthemed sound. As soon as there is a sound that matches in a theme, the search is stopped.

 To support localized sounds we first lookup the sound in the LC_MESSAGES locale setting of the program. If that fails, the locale string is truncated at the "@" if it includes it and it is tried again. Then, the locale string is truncated at the "_" if it includes it and it is tried again. Then it is looked for a sound in the "C" locale. Finally non-localized sound files are searched.

 If a sound name is not found, it is truncated at the last "-" and it is tried again. This is done again until no further "-" are present in the name string. This is useful to define common sounds for similar events. i.e. instead of defining two seperate sounds for "new-message-im" and "new-message-email" it might make sense to define just "new-message" instead.

 The lookup is done first in the requested output profile, followed by a lookup in "stereo" on failure and then without any output profile.

 The lookup algorithm should check for ".disabled" files first, followed by ".oga" (then ".ogg", although this might be removed later) and finally for ".wav".

 Configuration programs that allow limited user manipulation of the selected sound theme (i.e. for disabling or replacing certain sounds), should create a dynamicly created theme "__custom" that inherits from the selected theme and store it in the "~/.local/share/sounds/__custom" directory. Its index.theme should list a single directory ".". The sounds defined in that theme should not be attached to any output profile and should not be localized. The overwritten sounds should thus be stored directly below the aforementioned directory.

 The exact algorithm (in pseudocode) for looking up a sound in a theme is:

```

FindSound(sound, locale, outputprofile, theme) {
  filename = LookupSound (sound, locale, outputprofile, theme)
  if filename != none
    return filename

  if theme has parents
    for parent in theme.parents {
      filename = LookupSound (sound, locale, outputprofile, parent)
      if filename != none
        return filename
    }

  return none
}

```

 With the following helper functions:

```

LookupSound (requestedname, requestedlocale, requestedoutputprofile, requestedtheme) {
  for theme in (requestedtheme,
                "freedesktop",
                "") {
    for profile in (requestedoutputprofile, "stereo", "") {
      for each subdir in $(theme subdir list) {
        if DirectoryMatchesOutputProfile(subdir, profile) {
          for each directory in $(basename list) {
            for each name in (requestedname,
                              truncatesuffix(requestedname, "-"),
                              truncatesuffix(truncatesuffix(requestedname, "-"), "-"),
                              truncatesuffix(truncatesuffix(truncatesuffix(requestedname, "-"), "-"), "-"),
                              ...) {
              for each locale in (requestedlocale,
                                  truncatesuffix(requestedlocale, "@"),
                                  truncatesuffix(requestedlocale, "_"),
                                  "C",
                                  "") {
                for extension in ("disabled", "oga", "ogg", "wav") {
	          filename = directory/theme/subdir/locale/name.extension
                  if exist filename
                    return filename
                }
              }
            }
          }
        }
      }
    }
  }

  return none
}

DirectoryMatchesOutputProfile(subdir, profile) {
  read OutputProfile from subdir
  if OutputProfile == profile
    return true
  return false
}
```

## 6 Example [#](#example)

 Here is an example index.theme file:

```

[Sound Theme]
Name=Birch
Name[fr]=Bouleau
Comment=Sound theme using wooden instruments
Comment[fr]=Theme utilisant des instruments en bois
Inherits=wood,default
Directories=stereo 5.1

[stereo]
OutputProfile=stereo

[5.1]
OutputProfile=5.1
```

 The corresponding directory tree in the /usr/share/sounds directory could look like this:

```

birch/index.theme
birch/stereo/evolution-urgent-message.oga
birch/stereo/evolution-urgent-message.wav
birch/stereo/fr/evolution-urgent-message.oga
birch/stereo/evolution-urgent-message.sound
birch/5.1/evolution-urgent-message.oga
```

 Where birch/stereo/evolution-urgent-message.sound contains:

```

[Sound Data]
DisplayName=Evolution urgent message
DisplayName[fr]=Message urgent dans Evolution
```

 In this example a lookup of "evolution-urgent-message", with a 5.1 system preference and no localization, would get the "birch/5.1/evolution-urgent-message.oga" sound file.

## 7 Installing Application Sounds [#](#install_sounds)

 If you're an application author and you want to install sounds so that they can be used by your application, you should at least install the sound file in the "freedesktop" theme. This means installing a stereo WAV file in $XDG_DATA_DIRS/sounds/freedesktop/stereo/.

 Optionally you can install sounds in different formats and languages. For example, installing a localized OGG/Vorbis 5.1 game sound in $prefix/share/sounds/freedesktop/5.1/fr.

 You might even want to install sounds that match other well known themes so your application will fit in with some specific desktop environment.

 It is recommended that the sounds installed in the freedesktop theme sound neutral, since it is a fallback theme that will be used in combination with some very different sounds from any origin. But if you don't have any neutral sound, please install whatever sound you have in the freedesktop theme so that all your application can at least produce some sounds in all themes.

 After installing/updating a sound in a theme directory, make sure to update the mtime of the theme directory itself (i.e. touch $XDG_DATA_DIRS/sounds/freedesktop or similar), so that caches can be kept up-to-date.

## 8 Implementation Notes [#](#implementation_notes)

 The algorithm as described in this document works by always looking up filenames in directories (a stat in unix terminology). A good implementation is expected to read the directories once, and do all lookups in memory using that information.

 This caching can make it impossible for users to add sounds without having to restart applications. In order to handle this, any implementation that does caching is required to look at the mtime of the toplevel sound theme directories when doing a cache lookup, unless it already did so less than 5 seconds ago. This means that any sound editor or theme installation program may change the mtime of the the toplevel theme directory (such as $XDG_DATA_DIRS/sounds/freedesktop) where it changed the theme to make sure that the new sounds will eventually get used.

## 9 Background [#](#background)

 The sound theme specification is heavily based on the "Icon Theme Specification" by Alexander Larsson and Frans Englich.

## A Change history [#](#changes)

Version 0.5, 2 September 2008, Marc-Andre Lureau.

-

 Deprecate usage of ".ogg" extension in favor of ".oga" (according to [*MIME Types and File Extensions from xiph.org* (http://wiki.xiph.org/index.php/MIME_Types_and_File_Extensions)](http://wiki.xiph.org/index.php/MIME_Types_and_File_Extensions))

Version 0.4, 29 July 2008, Lennart Poettering.

-

 Document that every time a theme is updated to changed the directory needs to be touched.

Version 0.3, 25 July 2008, Lennart Poettering.

-

 Reordered lookup algorithm, to allow easy overwriting of sounds from inherited themes.

-

 Document "__custom" theme.

-

 Document ".disabled" sound files.

-

 Other fixes

Version 0.2, 4 June 2008, Marc-Andre Lureau and Lennart Poettering.

-

 Updated the Lookup algorithm.

-

 Remove Length and Locale from directory property.

-

 Use OutputProfile instead of SoundSystem.

-

 Add ReplayGain note.

Version 0.1, 11 February 2008, Marc-Andre Lureau.

-

 Initial draft.

© 2026 Freedesktop.org
