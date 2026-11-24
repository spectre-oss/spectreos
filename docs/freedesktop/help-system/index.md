[Jump to content](#_content)[Help System Specification](#index)[Show Contents: Help System Specification](#index)[Top](#)
1. [1 Overview](#overview)
2. [2 Directory Layout](#layout)
3. [3 URI Scheme](#uri-scheme)
4. [4 Help Formats](#formats)

## Help System Specification [#](#index)

Author: Shaun McCance `<shaunm@gnome.org>`
Publication Date: 2010-04-30

- **[1 Overview](#overview)**
- **[2 Directory Layout](#layout)**
- **[3 URI Scheme](#uri-scheme)**
- **[4 Help Formats](#formats)**

## 1 Overview [#](#overview)

This specification provides a common directory layout and URI scheme for locally installed help documents. Documents installed and referenced using this system will have better interoperability between different desktop environments and help applications.

## 2 Directory Layout [#](#layout)

A document is a collection of files, possibly from multiple directories in a path. Documents may be in any format, or even in multiple formats; see [Section 4, “Help Formats”](#formats) for details. A document has a *document identifier* that identifies it uniquely and specifies where it can be found on the file system. A document identifier consists of one or more characters from the following: digits (U+0030-U+0039), letters (U+0041-U+005A and U+0061-U+007A), hyphen (U+002D), underscore (U+005F), period (U+002E), and percent (U+0025).

Document identifiers are not explicitly namespaced. To avoid conflicts, document identifiers should start with the name of the application or package that provides the document. In many cases, the name of the application or package alone may be used. Otherwise, the document identifier should start with the name of the package or application followed by a hyphen.

Documents are installed under the `help` subdirectory in `$XDG_DATA_HOME` or in the `$XDG_DATA_DIRS` path. See the [XDG Base Directory Specification (http://standards.freedesktop.org/basedir-spec/latest/)](http://standards.freedesktop.org/basedir-spec/latest/) for details on `$XDG_DATA_HOME` and `$XDG_DATA_DIRS`. Each `help` directory in the path contains subdirectories for languages. Each language subdirectory contains subdirectories for documents, where the name of each subdirectory matches the document identifier of a document.

The *document path* for a given document is the list of directories of the following form:

```
datadir/help/lang/document
```

- **`*datadir*`**

Either `$XDG_DATA_HOME` or a directory in the `$XDG_DATA_DIRS` path.

- **`*lang*`**

The language code of a language in the user's list of preferred languages.

- **`*document*`**

The document identifier of the document.

The document path is ordered first according to the position of `*datadir*` in the path, then by the position of `*lang*` in the user's preferred list of languages. So, for example, if the user's preferred language list is `pt_BT:pt:C`, then `~/.local/share/help/pt/beanstalk` would take precedence over `/usr/share/help/pt_BR/beanstalk`.

## 3 URI Scheme [#](#uri-scheme)

Documents are referenced using the `help:` URI scheme. The `help:` URI scheme has the following form:

```
help:document/page?options#anchor
```

- ***document***

The document identifier.

- ***page***

An identifier for a page within the document. Documents often consist of multiple logical pages, which may not be reflected in the actual files on the system. The page identifier is optional. If it is not present, the preceding `/` character should be omitted.

- ***options***

Additional options that can influence how applications present a document. Options are optional. If they are not present, the preceding `?` character should be omitted. If they are present, they must conform to the `application/x-www-form-urlencoded` MIME type. Options may be used, for example, to override language settings or to provide keys for conditional processing. This specification makes no specific recommendations for the options.

- ***anchor***

An anchor point within a page. Applications should scroll to an appropriate point whenever possible. The anchor is optional. If it is not present, the preceding `#` character should be omitted.

Page identifiers and anchors may contain any character that is valid in a document identifier. Some document types may have further restrictions on page identifiers or anchors.

## 4 Help Formats [#](#formats)

Documents may be installed in any format. Not all help applications may recognize and handle the same types of documents. Whenever possible, help applications should support HTML. Documents may be installed in multiple formats. Help applications choose which format to use when multiple formats are present.

For any format, a help application must map the information found in the `help:` URI scheme to the information in that format. This specification contains recommendations for finding and handling documents in DocBook, Mallard, HTML, and XHTML.

### 4.1 DocBook [#](#docbook)

DocBook documents have a file named `index.docbook` in the document path. Any files referenced from the DocBook file, including XML fragments included through XInclude, should be looked up according to the document path.

When a help application displays a DocBook document, it will often display it in multiple pages or chunks, rather than as a single long page. The page identifier specifies which chunk to display. Not all applications will split a DocBook document into chunks in the same way, so the page identifier may not map exactly to the `id` attribute of an element that is chunked. In this case, the application displays the nearest enclosing chunk and treats the page identifier as an anchor if no anchor was explicitly provided.

### 4.2 Mallard [#](#mallard)

Mallard documents have a file named `index.page` in the document path. Other page files may be different directories in the document path. Any files referenced in any page file, including XML fragments included through XInclude, should be looked up according to the document path.

The page identifier specifies the id of a Mallard page file. The anchor specifies the id of a section within a Mallard page file.

### 4.3 HTML and XHTML [#](#html)

HTML documents have a file named `index.html` in the document path. XHTML documents have a file named `index.xhtml` in the document path. Other HTML or XHTML files in the document may be in different directories in the document path. Any files referenced in any HTML or XHTML page, including XML fragments included through XInclude in XHTML, should be looked up according to the document path.

The page identifier specifies the base file name, without the `.html` or `.xhtml` extension, of an HTML or XHTML file in the document. The anchor specifies a named anchor within the HTML or XHTML file.

© 2026 Freedesktop.org
