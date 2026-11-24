[Jump to content](#_content)[Thumbnail Managing Standard](#index)[Show Contents: Thumbnail Managing Standard](#index)[Top](#)
1. [1 History](#history)
2. [2 Introduction](#introduction)
3. [3 Issues To Solve](#issues)
4. [4 Thumbnail Directory](#directory)
5. [5 Thumbnail Creation](#creation)
6. [6 Thumbnail Saving](#thumbsave)
7. [7 Detect Modifications](#modifications)
8. [8 Thumbnail Creation Failures](#failures)
9. [9 Deleting Thumbnails](#delete)
10. [10 Shared Thumbnail repositories](#shared)
11. [11 Conclusion](#conclusion)
12. [12 Thanks](#thanks)
13. [13 Links](#links)

## Thumbnail Managing Standard [#](#index)

Authors: Jens Finke `<jens@gnome.org>`Olivier Sessink `<olivier@lx.student.wau.nl>`
Publication Date: May 2012, Version: Version 0.8.0

- **[1 History](#history)**
- **[2 Introduction](#introduction)**
- **[3 Issues To Solve](#issues)**
- **[4 Thumbnail Directory](#directory)**
- **[5 Thumbnail Creation](#creation)**
- **[6 Thumbnail Saving](#thumbsave)**
- **[7 Detect Modifications](#modifications)**
- **[8 Thumbnail Creation Failures](#failures)**
- **[9 Deleting Thumbnails](#delete)**
- **[10 Shared Thumbnail repositories](#shared)**
- **[11 Conclusion](#conclusion)**
- **[12 Thanks](#thanks)**
- **[13 Links](#links)**

## 1 History [#](#history)

-

December 2020, Version 0.9.0

  -

Added x-large and xx-large thumbnail sizes

-

May 2012, Version 0.8.0

  -

Modified to respect the [XDG Base Directory Specification (http://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)](http://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)

-

September 2004, Version 0.7.0

  -

Added readonly support for shared thumbnail repositories

-

September 2002, Version 0.6.1

  -

The subdirectories weren't a good idea. Removed them from this version.

  -

Updated link to the MD5 implementation.

-

September 2002, Version 0.6

  -

Added another sub directory level within the cache base directories to avoid too much clutter.

  -

State not to create thumbnails for files within the thumbnail cache directory.

  -

State when it's allowed to use thumbnails which haven't been checked for validity.

  -

Some typo fixes.

  -

Introduction and conclusion rewrite.

-

January 2002, Version 0.5

  -

Changed handling of different thumbnail sizes.

  -

Renamed directories.

  -

Propose using temporary filenames to avoid problems with concurrent access.

  -

Save thumbnails directly in the size dir without subdirs.

  -

Added optional Thumb::Mimetype key

  -

Give some more implementation notes.

  -

Added "Thanks" section.

-

December 2001, Version 0.4

  -

Destinction between required and optional thumbnail attributes.

  -

Dropped distinction between global/local .thumbnail directories.

  -

Use MD5 hashes as thumbnail filename.

  -

Initial attempt to handle concurrent accesses by different programs.

  -

Rewrote the "Deleting Thumbnails" section.

-

August 2001, Version 0.3

  -

Rewrote this paper.

-

July 2001, Version 0.2

  -

Removed distinction between low/high quality thumbnails.

  -

Separate directory for failures.

  -

Consider permission settings.

-

July 2001, Version 0.1

  -

First public release.

## 2 Introduction [#](#introduction)

 This paper deals with the permanent storage of previews for file content. In particular, it tries to define a general and widely accepted standard for this task. That way, it will be possible to share these so called thumbnails across a large number of applications.

The current situation is, that nearly every program introdues a new way of dealing with thumbnails. This results in the fact, that if the user uses 4 or 5 different programs, they will end up with 4 or 5 thumbnails for the same file. It's obvious that this is not only a waste of the user's disk space, but also makes the managing of large collections harder.

But why does a program use thumbnails? Often these are presented in file operation dialogs to give the user a hint what a certain file is about. This can be seen as information in addition to the plain filename which helps to identify the desired file faster and more easily. But the idea isn't limited to images and file operation dialogs. The additional value of previews is also applicable to other file types, like text documents, pdf files, spreadsheets and so on. The reason why this isn't deployed widely so far is, that it requires a lot of effort and is only of little use for a single program (for example, if only the spreadsheet program can create and view it's previews). But imagine if your filemanager could display all these previews too, while you are browsing through your filesystem.

If there is a general accepted, file type independent way how to deal with previews, the above sketched vision can come true. Every time an application saves a file it creates also a preview thumbnail for it. Other programs can check if there is a thumbnail for a specific file and can present it. This proposal tries to unify the thumbnail managing and constitutes the first step to a better graphical desktop.

## 3 Issues To Solve [#](#issues)

 There are some issues to solve to make this work correctly. Specifically these are:

-

Find a place for permanent storing.

-

Preserve information about original image and make them easily accessible without touching the original.

-

Provide the ability to handle different thumbnail sizes.

-

Take care of thumbnail generation failures.

-

Find a way to access thumbnails concurrently with different programs.

 All these things will be discussed in the next chapters and solutions will be presented.

## 4 Thumbnail Directory [#](#directory)

 For every user, there must be exactly one place where all generated thumbnails are stored. This thumbnails directory is located in the user's XDG Cache Home, as defined by the [XDG Base Directory Specification (http://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)](http://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html). Namely, if the environment variable $XDG_CACHE_HOME is set and not blank then the directory $XDG_CACHE_HOME/thumbnails will be used, otherwise $HOME/.cache/thumbnails will be used.

### 4.1 Directory Structure [#](#dirstructure)

 The thumbnails directory will have the following internal structure:

```

$XDG_CACHE_HOME/thumbnails/
$XDG_CACHE_HOME/thumbnails/normal
$XDG_CACHE_HOME/thumbnails/large/
$XDG_CACHE_HOME/thumbnails/x-large/
$XDG_CACHE_HOME/thumbnails/xx-large/
$XDG_CACHE_HOME/thumbnails/fail/
```

 The meaning of the directories are as follows:

-

Normal: The default place for storing thumbnails. The image size must not exceed 128x128 pixel. Programs which need smaller resolutions should read and write the 128x128 thumbnails and downscale them afterwards to the desired dimension. See [Thumbnail Creation](#creation) for more details.

-

Large: The previous notes apply to this directory too, except that the thumbnail size must be 256x256 pixel.

-

Extra Large: The previous notes apply to this directory too, except that the thumbnail size must be 512x512 pixel.

-

Extra extra Large: The previous notes apply to this directory too, except that the thumbnail size must be 1024x1024 pixel.

-

Fail: This directory is used to store information about files where a thumbnail couldn't be generated. See [Thumbnail Generation Failure](#failures) for more details.

###### Note

 You must not create/save thumbnails for any files you will find in these directories. Instead load and use these files directly.

## 5 Thumbnail Creation [#](#creation)

### 5.1 File format [#](#id-1.6.2)

 The image format for thumbnails is the [PNG format (http://www.w3.org/TR/REC-png)](http://www.w3.org/TR/REC-png), regardless in which format the original file was saved. To be more specific it must be a 8bit, non-interlaced PNG image with full alpha transparency (means 255 possible alpha values). However, every program should use the best possible quality when creating the thumbnail. The exact meaning of this is left to the specific program but it should consider applying antialiasing.

If the original file contains metadata affecting the interpretation of the image, it should be respected as much as possible. In particular, metadata specifying the orientation of the original image data should always be respected. The image data should be transformed as specified by the metadata before generating the thumbnail. JPEG files commonly have Exif orientation tags. TIFF files may also have Exif orientation tags, although this is less common. It is less critical, but still desirable, to respect other image metadata, such as white balance information.

### 5.2 Thumbnail Attributes [#](#addinfos)

Beside the storage of the raw graphic data its often useful to provide further information about a file in its thumbnail. Especially file size, image dimension or image type are often used in graphic programs. If the thumbnail provides such information it avoids any need to access the original file and thus makes the loading faster.

 The PNG format provides a mechanism to store arbitrary text strings together with the image. It uses a simple key/value scheme, where some keys are already predefined like Title, Author and so on (see [section 4.2.7 (http://www.w3.org/TR/REC-png#C.tEXt)](http://www.w3.org/TR/REC-png#C.tEXt) of the PNG standard). This mechanism is used to store additional thumbnail attributes.

 Beside the png format there is another internet standard which is important in this context: the [URI mechanism (https://www.ietf.org/rfc/rfc2396.txt)](https://www.ietf.org/rfc/rfc2396.txt). It is used to specify the location of the original file. For the global thumbnail repository, canonical absolute URIs (including the scheme) are used to determine the original uniquely. In shared thumbnail repositories, URIs are relative to the repository used.

The following keys and their appropriate values are used with this standard. All the keys are defined in the "Thumb::" namespace or if already defined by the PNG standard without any namespace.

###### Table 1: Used attributes. [#](#id-1.6.3.6.1)

| Key | Description | Global | Shared |
| --- | --- | --- | --- |
| Thumb::URI | The URI for the original file. For global thumbnails, this is an absolute canonical URI (e.g. file:///home/jens/photo/me.jpg). For shared thumbnail repositories, the URI is relative to the repository prefixed with ./ (e.g. ./picture.jpg). | Must verify | Must verify if present |
| Thumb::MTime | The modification time of the original file (as indicated by stat, which is represented as seconds since January 1st, 1970). | Must verify | Must verify if present |
| Thumb::Size | File size in bytes of the original file. | Should verify if present | Should verify if present |
| Thumb::Mimetype | The file mimetype. | Optional | Optional |
| Description | This key is predefined by the PNG standard. It provides a general description about the thumbnail content and can be used eg. for accessability needs. | Optional | Optional |
| Software | This key is predefined by the PNG standard. It stores the name of the program which generated the thumbnail. | Optional | Optional |

If it's not possible to obtain the modification time of the original then you shouldn't store a global thumbnail for it at all, but may use a shared thumbnail if it does not have a Thumb::MTime attribute. The mtime is needed to check if the thumbnail is valid yet (see [Detect modifications](#modifications)). Otherwise we can't guarantee the content corresponds to the original and must regenerate a thumb every time anyway.

There are surely some situations where further information are desired. Eg. the Gimp could save the number of layers an image has or something like this. So if an application wants to save more information it is free to do so. It should use a key in its own namespace (to avoid clashes) prefixed by X- to indicate that this is an extension. Eg. Gimp could save the layer info in the key X-GIMP::Layers.

 However, regarding to the filetype there are some keys which are generally useful. If a program can obtain information for the following keys it should provide them.

###### Table 2: Filetype specific attributes. [#](#id-1.6.3.10.1)

| Key | Description |
| --- | --- |
| Thumb::Image::Width | The width of the original image in pixel. |
| Thumb::Image::Height | The height of the original image in pixel. |
| Thumb::Document::Pages | The number of pages the original document has. |
| Thumb::Movie::Length | The length of the movie in seconds. |

With this approach a program doesn't have the guarantee that certain keys are stored in a thumbnail, because it may have been created by another application. If possible, a program should cope with the lack of information in such a case instead of recreating the thumbnail and the missing information.

### 5.3 Thumbnail Size [#](#thumbsize)

 As already mentionend in the [Thumbnail Directory](#directory) section there exists four suggested sizes you can use for your thumbnails: 128x128, 256x256, 512x512, 1024x1024 pixel. The idea is that if a program uses another size for it's previews it loads one of the two versions and scales them down to the desired size. Similar, when creating a thumbnail it scales the file down to 128x128 first (or 256x256, 512x512, or 1024x1024), saves it to disk and then reduce the size further. This mechanism enables all programs to obtain their desired previews in an easy and fast way.

 However, these are suggestions. Implementations should cope also with images that are smaller than the suggested size for the normal, large and extra large subdirectories. Depending on the difference between the actual and the desired size, they can either use the smaller one found in the cache and scale it down or recreate a thumbnail with the proposed size for this directory.

 If a program needs a thumbnail for an image file which is smaller than 128x128 pixel it doesn't need to save it at all.

###### Note

 All sizes define just a rectangle area where the thumbnail must fit in. Don't scale every image to a rectangular thumbnail but preserve the ratio instead!

## 6 Thumbnail Saving [#](#thumbsave)

The thumbnail filename is determined by a hashfunction. This proposal utilizes [MD5 (https://www.ietf.org/rfc/rfc1321.txt)](https://www.ietf.org/rfc/rfc1321.txt) as hash mechanism in the following way.

1.

For global thumbnails, you need the absolute canonical URI for the original file, as stated in [ URI RFC 2396 (https://www.ietf.org/rfc/rfc2396.txt)](https://www.ietf.org/rfc/rfc2396.txt). In particular this defines to use three '/' for local 'file:' resources (see example below). For shared thumbnails, you need a canonical URI relative to the shared thumbnail repository, comprised of a single "./"-prefixed and properly encoded path segment for the filename, e.g. "./picture.png". The "./" prefix is required to simplify encoding, and use of the canonical, minimally encoded form of the URI is required to avoid mismatches between thumbnail generator and thumbnail reader.

2.

Calculate the MD5 hash for this URI. Not for the file it points to! This results in a 128bit hash, which is representable by a hexadecimal number in a 32 character long string.

3.

To get the final filename for the thumbnail just append a '.png' to the hash string. According to the dimension of the thumbnail you must store the result either in $XDG_CACHE_HOME/thumbnails/normal, $XDG_CACHE_HOME/thumbnails/large, $XDG_CACHE_HOME/thumbnails/x-large, or $XDG_CACHE_HOME/thumbnails/xx-large.

 An example will illustrate this:

###### Example 1: Saving a global thumbnail [#](#id-1.7.5)

 Consider we have a file ~/photos/me.png. We want to create a thumbnail with a size of 128x128 pixel for it, which means it will be stored in the $XDG_CACHE_HOME/thumbnails/normal directory. The absolute canonical URI for the file in this example is file:///home/jens/photos/me.png.

The MD5 hash for the uri as a hex string is c6ee772d9e49320e97ec29a7eb5b1697. Following the steps above this results in the following final thumbnail path:

```

/home/jens/.cache/thumbnails/normal/c6ee772d9e49320e97ec29a7eb5b1697.png
```

###### Example 2: Saving a shared thumbnail [#](#id-1.7.6)

 Consider we have a file /mnt/pictures/picture.png. We want to create a thumbnail with a size of 128x128 pixel for it, which means it will be stored in the /mnt/pictures/.sh_thumbnails/normal directory. The relative canonical URI for the file in this example is ./picture.png.

 The MD5 hash for the uri as a hex string is 7fd0e41c1612f860427a76c4100745a3. Following the steps above this results in the following final thumbnail path:

```

/mnt/pictures/.sh_thumbnails/normal/7fd0e41c1612f860427a76c4100745a3.png
```

### 6.1 Permissions [#](#id-1.7.7)

A few words regarding permissions: All the directories including the $XDG_CACHE_HOME/thumbnails directory must have set their permissions to 700 (this means only the owner has read, write and execute permissions, see "man chmod" for details). Similar, all the files in the thumbnail directories should have set their permissions to 600. This way we assure that if a user creates a thumbnail for a file where only he has read-permissions no other user can take a glance on it through the backdoor with the thumbnails.

Programs should first check that the original image file is readable. If it is not, the program should not attempt to read a thumbnail from the cache, and it should not save any information in the cache (including "failed" thumbnails). Otherwise, thumbnailing will be prevented even if the permissions are changed to permit reading.

### 6.2 Concurrent Thumbnail Creation [#](#id-1.7.8)

An important goal of this paper is to enable programs to share their thumbnails. This includes the occurrences of concurrent accesses to the cache by different programs. Problems arise if two programs try to create a thumbnail for the same file at the same time. Because of this the following procedure is suggested:

1.

Check if the thumbnail already exists and if it's valid.

2.

If the above conditions are not fulfilled create the thumbnail and write it under a temporary filename onto the disk.

3.

Rename the temporary file to the thumbnail filename. Since this is an atomic operation the new thumbnail is either completely written or not.

 This way the worst case is that a thumbnail will be written twice. However, the thumbnail is in a sane state at any time.

###### Note

 The temporary file should be placed into the same directory as the final thumbnail, because then you are sure that they lay on the same filesystem. This guarantees a fast renaming of the temporary file. Using a combination of programname, process id and eg. first characters from the hash string should give a fairly unique temporary name.

### 6.3 Advantages Of This Approach [#](#id-1.7.9)

Previously versions of this standard used a very different mechanism for storing thumbnails. But this one has some very important advantages:

1.

Works for all kinds of possible file locations, since its based only on the textual URI representation of a file. This way files that are located on the locale filesystem or a samba, http, ftp or WebDAV server can be treated equally.

2.

It results in a flat directory hierarchy which assures fast access. Since the hash is always 32 characters long the thumbnail filename is exactly 36 characters long for every possible file (including the '.png' suffix).

3.

Due to the usage of the MD5 hash its unlikely that there occur clashes between two different thumbnails, even if it's theoretically possible. But the probability is very low and can be ignored in this context. The worst case would be that a thumbnail overwrites another valid one. Ok, if they have exactly the same modification time it is possible too that a wrong thumbnail for a file will be displayed (see [Detect Modifications](#modifications)).

4.

It's very easy to implement.

###### Note

There do exist a lot of different library implementations for the MD5 hash algorithm. If you don't want to add yet another library dependency to support thumbnailing in your program you can eg. use the [RFC 1321 implementation (ftp://mirror.cs.wisc.edu/pub/mirrors/ghost/packages/md5.tar.gz)](ftp://mirror.cs.wisc.edu/pub/mirrors/ghost/packages/md5.tar.gz) by L. Peter Deutsch. It adds only 1.5kb sourcecode in two files to your project and can be used without much restrictions.

## 7 Detect Modifications [#](#modifications)

 One important thing is to assure that the thumbnail image displays the same information than the original, only in a downscaled version. To make this possible we compare the original file's size and modification time with the attribytes stored in the ['Thumb::MTime'](#addinfos) and ['Thumb::Size'](#addinfos) keys. The absence of the MTime key in a global thumbnail is an error; apart from that, checks are only performed if the relevant key is present. If any check fails, the thumbnail can not be used and must be recreated.

###### Example 3: Algorithm to check for modification. [#](#id-1.8.2.3)

```

if ((!thumb.isShared && !isSet(thumb.MTime)) ||
      (isSet(thumb.MTime) && file.mtime != thumb.MTime) ||
      (isSet(thumb.Size) && file.size != thumb.Size) {
   recreate_thumbnail();
}
```

 Relying solely on modification times may fail in cases where collections of files have been copied to a new folder. several files can share the same modification time - to within the one second resolution of ['Thumb::MTime'](#addinfos). Activities like swapping the filenames of these files may then result in incorrect thumbnails. Where the thumbnail includes the ['Thumb::Size' key](#addinfos), the extra check of comparing sizes can avoid this issue.

###### Note

 It is not sufficient to do a `file.mtime > thumb.MTime` check. If the user moves another file over the original, where the mtime changes but is in fact lower than the thumbnail stored mtime, we won't recognize this modification.

 If for some reason a non-shared thumbnail doesn't have the 'Thumb::MTime' key (although it's required) it should be recreated in any case.

###### Note

There are certain circumstances where a program can't or don't want to update a thumbnail (eg. within a history view of your recently edited files). This is legally but it should be indicated to the user that an thumbnail is maybe outdated or not even checked for modifications.

## 8 Thumbnail Creation Failures [#](#failures)

Due to several reasons its possible that the generation of a thumbnail fails:

-

The file format is unknown and cannot be loaded by the program.

-

The file format is known but the file is somehow broken and thus cannot be read.

-

 The generation of a thumbnail would take too long, due to the large size of the file.

Under some circumstances a program want to preserve the information that the creation failed. Eg to avoid trying it again and again in the future. The problem is that the above mentioned issues are often program specific. Eg Nautilus can't read the native Gimp format xcf but of course Gimp can and could create thumbnails for them. Or one program uses a broken TIFF implementation which refuses to load an TIFF image but another one uses a correct implementation.

 Because of this, its best to save these failure information per program. In the [Directory Structure](#dirstructure) section there is already a 'fail' directory mentioned, which should be used for this. Every program must create a directory of it's own there with the name of the program appended by the version number (eg. `$XDG_CACHE_HOME/thumbnails/fail/nautilus-1.0`).

 For every thumbnail generation failure of a readable image, the program creates an empty PNG file. If it's possible to obtain some additional information from the image (see [Store Additional Information](#addinfos)) they should be stored together with the thumbnail too, at least the required 'Thumb::MTime' and 'Thumb::URI' keys must be set. The procedure for the saving of such a fail image is the same as described in [Thumbnail Saving](#thumbsave). You must only use the application specific directory within `$XDG_CACHE_HOME/thumbnails/fail` instead of the size specific ones.

This approach has the advantage that a program can access information about a thumbnail creation failure the same way as it does with successfully generated ones.

## 9 Deleting Thumbnails [#](#delete)

 The deletion of a thumbnail is somehow tricky. A general rule is that a thumbnail should be deleted if the original file doesn't exist anymore (Note: If it was only modified the thumbnail should be recreated instead). There are different ways how this can be achieved:

1.

If a file manager is aware of this standard and deletes a file it could take care of deleting the thumbnail too.

2.

A daemon runs in the background which cleans up the cache in certain intervals.

3.

The user can call a managing tool which lists all the thumbnails together with their original file paths. From there they can delete single images, all images where the original doesn't exist anymore or all images older than eg. 30 days.

Another problem is that there are some URI schemes where it isn't directly possible to determine if the file exists or not. Eg. this applies to all the internet related schemes like http:, ftp: and so on when you don't have an internet connection. The same applies to removable media eg. a cdrom.

The above mentioned managing tools should therefore consider the following rules:

1.

If the URI scheme specifies a local file (like the file: scheme) then it should check if the original file exists. If it doesn't exist anymore the program should delete the thumbnail.

2.

For all internet related schemes (like http: or ftp:) delete the thumbnail if it hasn't been accessed within a certain user defined time period (can default to 30 days).

3.

Removable media should be considered too. Although this can't work for all systems in all cases reliable there are some heuristics which can be used. Eg. checking the fstab configuration file and look for the mount point of /dev/fd0 (floppy disk) or check if the CD-Rom drive is mounted under /cdrom. Thumbnails for removable media files should be handled as in the previous point.

## 10 Shared Thumbnail repositories [#](#shared)

 In some situations it is desirable to have a shared thumbnail repository. This is a read-only collection of thumbnails that is shared among different users or different computers. For example a CD-ROM with images, could include the thumbnails for these images such that they do not need to be generated for every user or computer accessing this CD-ROM.

 A shared thumbnail repository is stored in the directory whose files it should contain thumbnails for. The location for a shared thumbnail repository inside such a directory will be:

```
.sh_thumbnails/
```

Within this directory are the same subdirectories as in the global thumbnail directory.

```

.sh_thumbnails/
.sh_thumbnails/normal/
.sh_thumbnails/large/
.sh_thumbnails/x-large/
.sh_thumbnails/xx-large/
.sh_thumbnails/fail/

```

The meaning of these directories is identical to their meaning in the global directory.

 Shared thumbnails use URIs relative to the thumbnail respository instead of absolute canonical URIs. This applies both to the URI used when generating the thumbnail filename using md5sum, as well as the Thumb::URI property. This is due to absolute paths not necessarily being consistent across different users of the shared thumbnail repository.

 Given an original file at "/mnt/pictures/picture.png", the shared thumbnail repository would be located at "/mnt/pictures/.sh_thumbnails/". The repository would contain an entry for the URI "./picture.png". A relative thumbnail URI must be comprised "./" followed by a single path segment for the filename with canonical URI encoding applied.

 For shared repositories whose access method does not provide a consistent mtime (or not even a consistent name), the creator may leave out the Thumb::MTime (or Thumb::URI) metadata items, as long as it has alternative means of determining freshness and removing stale thumbnail files. For example, when a shared thumbnail generator detects that a raw file and a compressed file are visually identical, it can use a hardlink to a single thumbnail file for two files that vary in their MTime, file name (thus URI) and size, as long as it includes neither of those metadata in the thumbnail file.

### 10.1 Creating thumbnails in a shared thumbnail repository [#](#id-1.11.11)

 A shared thumbnail repository should be considered read-only. A program should never add or update a thumbnail in the shared thumbnail repository. Such a repository should only be created on special request by the user. If a thumbnail is outdated or corrupt, a program should create a new thumbnail in the personal thubmnail repository, and not update the shared thumbnail repository.

 If the user specific requested the creation of a shared thumbnail repository, the thumbnails can be created. Because the URI for shared images is possibly not constant, this means that the full URI can not be stored in the thumbnail. The URI field should, therefore, contain only the filename, and no directory parts. All other properties, however, should be the same as in the personal repository, including the size. The permissions for shared thumbnails should be the same as their original images.

### 10.2 Loading thumbnails from a shared thumbnail repository [#](#id-1.11.12)

 When loading thumbnails from a shared thumbnail repository, the personal repository has a higher priority. If a thumbnail exists in the personal thumbnail repository, this thumbnail should be used, and not the thumbnail from the shared repository.

 There is one exception to this rule. If the thumbnail in the personal thumbnail repository is outdated or corrupt, the thumbnail from the shared repository should be checked. If this thumbnail is correct, the thumbnail in the personal repository can be deleted and the thumbnail from the shared collection can be used.

The lack of Thumb::MTime and Thumb::URI in a shared thumbnail does not automatically mark the thumbnail as invalid. The program may still opt to create a local thumbnail, but may prefer not to do so (e.g. because it would involve heavy network traffic).

## 11 Conclusion [#](#conclusion)

 The proposed way of dealing with file previews fulfiles the requirements of a file type independent preview cache. It is relative easy to use, understand and implement. All these are important facts to allow it's wide spread.

The next step will be to take these ideas to the applications. If a lot of users, coders and maintainers will cooperate on this, we can reach a new level of usability.

## 12 Thanks [#](#thanks)

The following people helped me to write this paper with a lot of suggestions, good ideas and constructive critism. They found serious bugs and problems in previous versions or helped me in another way. Thank you very much:

 Darin Adler (Gnome/Nautilus), Alexander Larsson (Gnome/Nautilus), Thomas Leonard (Rox Desktop), Sven Neumann (Gimp), Havoc Pennington (Gnome/freedesktop.org), Malte Starostik (KDE), Owen Taylor (GTK), and all I forgot to mention here.

## 13 Links [#](#links)

-

URI standard: [https://www.ietf.org/rfc/rfc2396.txt (https://www.ietf.org/rfc/rfc2396.txt)](https://www.ietf.org/rfc/rfc2396.txt)

-

PNG standard: [http://www.w3.org/TR/REC-png (http://www.w3.org/TR/REC-png)](http://www.w3.org/TR/REC-png)

-

MD5 hash algorithm: [https://www.ietf.org/rfc/rfc1321.txt (https://www.ietf.org/rfc/rfc1321.txt)](https://www.ietf.org/rfc/rfc1321.txt)

© 2026 Freedesktop.org
