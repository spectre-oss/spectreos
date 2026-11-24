[Jump to content](#_content)[Recent File Storage Specification](#index)[Show Contents: Recent File Storage Specification](#index)[Top](#)
1. [1 Overview](#overview)
2. [2 Storage](#storage)
3. [3 Notification](#notification)
4. [A Change history](#changes)

## Recent File Storage Specification [#](#index)

Author: James Willcox `<jwillcox@gnome.org>`
Publication Date: 12 August 2002, Version: Version 0.2

- **[1 Overview](#overview)**
- **[2 Storage](#storage)**
- **[3 Notification](#notification)**
- **[A Change history](#changes)**

## 1 Overview [#](#overview)

 Many applications choose to have a menu containing a list of recently used files, which can be used as an alternative to opening a file selector. GNOME, and KDE applications have done this for quite some time, and probably others have as well. This specification aims to do the following things:

-

Provide a standard mechanism for storing a list of recently used files (really, URIs)

-

Allow for notification of changes in the list

 Accomplishing this will make using applications that are not native to the user's desktop a bit nicer, and give a more unified feel to the X desktop in general. It will be especially useful to applications that don't belong to a specific desktop at all, such as Mozilla and Open Office, since those applications could become compatible with all desktops in this area without special case considerations for each one.

## 2 Storage [#](#storage)

 For different desktops and applications to have access to the same information, a protocol for storing the recent file list has to be determined. An XML document will be used, of the form below:

```

<?xml version="1.0"?>
<RecentFiles>
  <RecentItem>
    <URI>file:///home/jwillcox/testfile.txt</URI>
    <Mime-Type>text/plain</Mime-Type>
    <Timestamp>1028181153</Timestamp>
    <Private/>
    <Groups>
      <Group>Recent File Test</Group>
    </Groups>
  </RecentItem>
  <RecentItem>
    <URI>file:///home/jwillcox/recent-file-spec.xml</URI>
    <Mime-Type>text/xml</Mime-Type>
    <Timestamp>1028181158</Timestamp>
    <Private/>
    <Groups>
      <Group>Recent File Test</Group>
    </Groups>
  </RecentItem>
</RecentFiles>

```

 The URI, Mime-Type, Timestamp tags are required, but the Private, Groups, and Group tags are not. The Timestamp tag should be the number of seconds since the Epoch when the item was added to the list. The Group tags exist for the purpose of making groups of items with an arbitrary set of MIME types. A single item can belong to an unlimited number of groups. If the Private tag is specified, it means that implementations should only include the item if requestor has specifically asked for items in a group that the item belongs to. For instance, maybe a file selector would like to hold a list of directories that you have recently saved files in, and allow you to choose from one of these next time you save a file. You wouldn't want those directories showing up in the "Recent Documents" menu. If the items are put into the "File Selector Save Directories" group and marked as private, this problem is solved.

 All text in the file should be stored in the UTF-8 encoding. No local paths are allowed in the URI tag. They should be converted to a valid [URI (http://www.ietf.org/rfc/rfc2396.txt)](http://www.ietf.org/rfc/rfc2396.txt) with a "file" scheme. Items with the same URI should not be allowed. If a duplicate is found, it should have its timestamp updated, and any new groups added to it. No other information should be changed. The document should be stored in "~/.recently-used", and it should contain no more than 500 items.

 Before reading or writing the document, it should be locked using the POSIX lockf() mechanism, and unlocked when finished.

## 3 Notification [#](#notification)

 Notification should be accomplished by simply monitoring the document for changes. This can be done by either polling the file every so often, or using a library like [FAM (http://oss.sgi.com/projects/fam/)](http://oss.sgi.com/projects/fam/).

## A Change history [#](#changes)

Version 0.2, 12 August 2002, James Willcox.

-

 Switched from flat-file storage to XML. This was done to facilitate easier storage of meta-data. Files can now belong to "groups", and the timestamp is now a required piece of meta-data.

Version 0.1, 8 July 2002, James Willcox.

-

 Created initial draft.

© 2026 Freedesktop.org
