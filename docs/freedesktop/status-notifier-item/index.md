[Jump to content](#_content)[Status Notifier Item Specification](#index)[Show Contents: Status Notifier Item Specification](#index)[Top](#)
1. [1 Introduction](#introduction)
2. [2 Basic Design](#basic-design)
3. [3 StatusNotifierItem](#status-notifier-item)
4. [4 StatusNotifierWatcher](#status-notifier-watcher)
5. [5 StatusNotifierHost](#status-notifier-host)
6. [6 Icon pixmap](#icon-pixmap)
7. [7 Markup](#markup)

## Status Notifier Item Specification [#](#index)

- File Name: status-notifier-item-spec.xml
- ID: no ID found

Authors: Marco Martin `<mart@kde.org>`Aaron Seigo `<aseigo@kde.org>`
Publication Date: TBD, Version: Version 0.1

- **[1 Introduction](#introduction)**
- **[2 Basic Design](#basic-design)**
- **[3 StatusNotifierItem](#status-notifier-item)**
- **[4 StatusNotifierWatcher](#status-notifier-watcher)**
- **[5 StatusNotifierHost](#status-notifier-host)**
- **[6 Icon pixmap](#icon-pixmap)**
- **[7 Markup](#markup)**

## 1 Introduction [#](#introduction)

- File Name: status-notifier-item-spec.xml
- ID: introduction

 This specification defines the management of visual items, usually icons used for reporting the status of an application to the user or provide quick access to common actions performed by that application. It is intended to be complementary but not directly related with the Freedesktop's Desktop Notifications specification and is aimed as a replacement to the Freedesktop System tray specification, though more model-view oriented, giving more freedom to the workspace how to graphically represent the items coherent to its visual style language.

 This specification does not define what the aspect of the Notification Items will be, this is strictly implementation specific.

 Example use cases include:

-

Mixer volume control.

-

Battery indicator.

-

Instant messenger on-line status.

-

Generic application information and actions, like media player controls.

## 2 Basic Design [#](#basic-design)

- File Name: status-notifier-item-spec.xml
- ID: basic-design

 The Status Notifier Item system relies on inter-process communication via D-Bus and is composed of three parts:

-

[`StatusNotifierItem`](#status-notifier-item): Each application that wants to utilize status notifiers registers its own `StatusNotifierItem` to the session bus.

-

[`StatusNotifierWatcher`](#status-notifier-watcher): Service used to keep track of every active instance of `StatusNotifierItem`, used by StatusNotifierHost to get the list of all items. It also notifies it when items are added and removed.

-

[`StatusNotifierHost`](#status-notifier-host): An application that wants to provide a visualization of the existing `StatusNotifierItem` instances must register a `StatusNotifierHost` in the session bus.

## 3 StatusNotifierItem [#](#status-notifier-item)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item

 Each application can register an arbitrary number of Status Notifier Items by registering on the session bus the service `org.freedesktop.StatusNotifierItem-PID-ID`, where `PID` is the process id of the application and `ID` is an arbitrary numeric unique identifier between different instances registered by the same application.

 As soon as a new instance of a `StatusNotifierItem` is created, the application must register the unique instance name to the [StatusNotifierWatcher](#status-notifier-watcher).

 Each instance of StatusNotifierItem must provide an object called StatusNotifierItem with the following properties, methods and signals described in the following sections.

### 3.1 Properties [#](#status-notifier-item-properties)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties

#### 3.1.1 org.freedesktop.StatusNotifierItem.Category [#](#status-notifier-item-properties-category)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-category

`STRING org.freedesktop.StatusNotifierItem.Category ();`

 Describes the category of this item.

 The allowed values for the `Category` property are:

-

`ApplicationStatus`: The item describes the status of a generic application, for instance the current state of a media player. In the case where the category of the item can not be known, such as when the item is being proxied from another incompatible or emulated system, ApplicationStatus can be used a sensible default fallback.

-

`Communications`: The item describes the status of communication oriented applications, like an instant messenger or an email client.

-

`SystemServices`: The item describes services of the system not seen as a stand alone application by the user, such as an indicator for the activity of a disk indexing service.

-

`Hardware`: The item describes the state and control of a particular hardware, such as an indicator of the battery charge or sound card volume control.

#### 3.1.2 org.freedesktop.StatusNotifierItem.Id [#](#status-notifier-item-properties-id)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-id

`STRING org.freedesktop.StatusNotifierItem.Id ();`

It's a name that should be unique for this application and consistent between sessions, such as the application name itself.

#### 3.1.3 org.freedesktop.StatusNotifierItem.Title [#](#status-notifier-item-properties-title)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-title

`STRING org.freedesktop.StatusNotifierItem.Title ();`

It's a name that describes the application, it can be more descriptive than `Id`.

#### 3.1.4 org.freedesktop.StatusNotifierItem.Status [#](#status-notifier-item-properties-status)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-status

`STRING org.freedesktop.StatusNotifierItem.Status ();`

 Describes the status of this item or of the associated application.

 The allowed values for the `Status` property are:

`Passive`: The item doesn't convey important information to the user, it can be considered an "idle" status and is likely that visualizations will chose to hide it.

`Active`: The item is active, is more important that the item will be shown in some way to the user.

`NeedsAttention`: The item carries really important information for the user, such as battery charge running out and is wants to incentive the direct user intervention. Visualizations should emphasize in some way the items with NeedsAttention status.

#### 3.1.5 org.freedesktop.StatusNotifierItem.WindowId [#](#status-notifier-item-properties-windowid)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-windowid

`UINT32 org.freedesktop.StatusNotifierItem.WindowId ();`

 It's the windowing-system dependent identifier for a window, the application can chose one of its windows to be available through this property or just set 0 if it's not interested.

#### 3.1.6 org.freedesktop.StatusNotifierItem.IconName [#](#status-notifier-item-properties-iconname)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-iconname

`STRING org.freedesktop.StatusNotifierItem.IconName ();`

 The StatusNotifierItem can carry an icon that can be used by the visualization to identify the item.

 An icon can either be identified by its Freedesktop-compliant icon name, carried by this property of by the icon data itself, carried by the property `IconPixmap`. Visualizations are encouraged to prefer icon names over icon pixmaps if both are available (FIXME: still not very defined: could the pixmap be used as fallback if an icon name is not found?)

#### 3.1.7 org.freedesktop.StatusNotifierItem.IconPixmap [#](#status-notifier-item-properties-iconpixmap)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-iconpixmap

`ARRAY(INT, INT, ARRAY BYTE) org.freedesktop.StatusNotifierItem.IconPixmap ();`

[Icon pixmap](#icon-pixmap) to use for displaying the icon of the item.

#### 3.1.8 org.freedesktop.StatusNotifierItem.OverlayIconName [#](#status-notifier-item-properties-overlayiconname)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-overlayiconname

`STRING org.freedesktop.StatusNotifierItem.OverlayIconName ();`

 The Freedesktop-compliant name of an icon. This can be used by the visualization to indicate extra state information, for instance as an overlay for the main icon.

#### 3.1.9 org.freedesktop.StatusNotifierItem.OverlayIconPixmap [#](#status-notifier-item-properties-overlayiconpixmap)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-overlayiconpixmap

`ARRAY(INT, INT, ARRAY BYTE) org.freedesktop.StatusNotifierItem.OverlayIconPixmap ();`

[Icon pixmap](#icon-pixmap) to use for the [overlay icon](#status-notifier-item-properties-overlayiconname).

#### 3.1.10 org.freedesktop.StatusNotifierItem.AttentionIconName [#](#status-notifier-item-properties-attentioniconname)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-attentioniconname

`STRING org.freedesktop.StatusNotifierItem.AttentionIconName ();`

 The Freedesktop-compliant name of an icon. this can be used by the visualization to indicate that the item is in `RequestingAttention` state.

#### 3.1.11 org.freedesktop.StatusNotifierItem.AttentionIconPixmap [#](#status-notifier-item-properties-attentioniconpixmap)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-attentioniconpixmap

`ARRAY(INT, INT, ARRAY BYTE) org.freedesktop.StatusNotifierItem.AttentionIconPixmap ();`

[Icon pixmap](#icon-pixmap) to use for the [attention icon](#status-notifier-item-properties-attentioniconname).

#### 3.1.12 org.freedesktop.StatusNotifierItem.AttentionMovieName [#](#status-notifier-item-properties-attentionmoviename)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-attentionmoviename

`STRING org.freedesktop.StatusNotifierItem.AttentionMovieName ();`

 An item can also specify an animation associated to the RequestingAttention state. This should be either a Freedesktop-compliant icon name or a full path. The visualization can chose between the movie or `AttentionIconPixmap` (or using neither of those) at its discretion.

#### 3.1.13 org.freedesktop.StatusNotifierItem.ToolTip [#](#status-notifier-item-properties-tooltip)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-tooltip

`(STRING, ARRAY(INT, INT, ARRAY BYTE), STRING, STRING) org.freedesktop.StatusNotifierItem.ToolTip ();`

 Data structure that describes extra information associated to this item, that can be visualized for instance by a tooltip (or by any other mean the visualization consider appropriate. Components are:

 STRING: Freedesktop-compliant name for an icon.

 ARRAY(INT, INT, ARRAY BYTE): icon data

 STRING: Title for this tooltip

 STRING: Descriptive text for this tooltip. It can contain also a subset of the HTML markup language, for a list of allowed tags see [Markup](#markup).

#### 3.1.14 org.freedesktop.StatusNotifierItem.ItemIsMenu [#](#status-notifier-item-properties-itemismenu)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-itemismenu

`BOOL`: The item only support the context menu, the visualization should prefer showing the menu or sending `ContextMenu()` instead of `Activate()`

#### 3.1.15 org.freedesktop.StatusNotifierItem.Menu [#](#status-notifier-item-properties-menu)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-properties-menu

 OBJECT PATH: D-Bus path to an object which should implement the com.canonical.dbusmenu interface

### 3.2 Methods [#](#status-notifier-item-methods)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-methods

#### 3.2.1 ContextMenu [#](#status-notifier-item-methods-contextmenu)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-methods-contextmenu

`VOID org.freedesktop.StatusNotifierItem.ContextMenu (INT x, INT y);`

Asks the status notifier item to show a context menu, this is typically a consequence of user input, such as mouse right click over the graphical representation of the item.

The x and y parameters are in screen coordinates and is to be considered an hint to the item about where to show the context menu.

#### 3.2.2 Activate [#](#status-notifier-item-methods-activate)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-methods-activate

`VOID org.freedesktop.StatusNotifierItem.Activate (INT x, INT y);`

Asks the status notifier item for activation, this is typically a consequence of user input, such as mouse left click over the graphical representation of the item. The application will perform any task is considered appropriate as an activation request.

The x and y parameters are in screen coordinates and is to be considered an hint to the item where to show eventual windows (if any).

#### 3.2.3 SecondaryActivate [#](#status-notifier-item-methods-secondaryactivate)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-methods-secondaryactivate

`VOID org.freedesktop.StatusNotifierItem.SecondaryActivate (INT x, INT y);`

Is to be considered a secondary and less important form of activation compared to Activate. This is typically a consequence of user input, such as mouse middle click over the graphical representation of the item. The application will perform any task is considered appropriate as an activation request.

The x and y parameters are in screen coordinates and is to be considered an hint to the item where to show eventual windows (if any).

#### 3.2.4 Scroll [#](#status-notifier-item-methods-scroll)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-methods-scroll

`VOID org.freedesktop.StatusNotifierItem.Scroll (INT delta, STRING orientation);`

The user asked for a scroll action. This is caused from input such as mouse wheel over the graphical representation of the item.

The delta parameter represent the amount of scroll, the orientation parameter represent the horizontal or vertical orientation of the scroll request and its legal values are horizontal and vertical.

### 3.3 Signals [#](#status-notifier-item-signals)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-signals

#### 3.3.1 NewTitle [#](#status-notifier-item-signals-newtitle)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-signals-newtitle

`VOID org.freedesktop.StatusNotifierItem.NewTitle ();`

The item has a new title: the graphical representation should read it again immediately.

#### 3.3.2 NewIcon [#](#status-notifier-item-signals-newicon)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-signals-newicon

`VOID org.freedesktop.StatusNotifierItem.NewIcon ();`

The item has a new icon: the graphical representation should read it again immediately.

#### 3.3.3 NewAttentionIcon [#](#status-notifier-item-signals-newattentionicon)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-signals-newattentionicon

`VOID org.freedesktop.StatusNotifierItem.NewAttentionIcon ();`

The item has a new attention icon: the graphical representation should read it again immediately.

#### 3.3.4 NewOverlayIcon [#](#status-notifier-item-signals-newoverlayicon)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-signals-newoverlayicon

`VOID org.freedesktop.StatusNotifierItem.NewOverlayIcon ();`

The item has a new overlay icon: the graphical representation should read it again immediately.

#### 3.3.5 NewToolTip [#](#status-notifier-item-signals-newtooltip)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-signals-newtooltip

`VOID org.freedesktop.StatusNotifierItem.NewToolTip ();`

The item has a new tooltip: the graphical representation should read it again immediately.

#### 3.3.6 NewStatus [#](#status-notifier-item-signals-newstatus)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-item-signals-newstatus

`VOID org.freedesktop.StatusNotifierItem.NewStatus (STRING status);`

The item has a new status, that is passed as an argument of the signal.

## 4 StatusNotifierWatcher [#](#status-notifier-watcher)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher

 There will be a single `org.freedesktop.StatusNotifierWatcher` service instance registered on the session but at any given time. The StatusNotifierWatcher service is used to keep track of StatusNotifierItem instances, enumerate them and notify when new ones are registered or old ones are unregistered.

 It is also used to keep track of org.freedesktop.StatusNotifierHost instances, to have an easy way to know if there is at least one service registered as the visualization host for the status notifier items.

### 4.1 Properties [#](#status-notifier-watcher-properties)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-properties

#### 4.1.1 RegisteredStatusNotifierItems [#](#status-notifier-watcher-properties-registered-status-notifier-items)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-properties-registered-status-notifier-items

`ARRAY(STRING) org.freedesktop.StatusNotifierWatcher.RegisteredStatusNotifierItems ();`

List containing all the registered instances of StatusNotifierItem. All elements of the array should correspond to services actually running on the session bus at the moment of the method call.

#### 4.1.2 IsStatusNotifierHostRegistered [#](#status-notifier-watcher-properties-is-status-notifier-host-registered)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-properties-is-status-notifier-host-registered

`BOOL org.freedesktop.StatusNotifierWatcher.IsStatusNotifierHostRegistered ();`

True if at least one `StatusNotifierHost` has been registered with [RegisterStatusNotifierHost](#status-notifier-watcher-methods-register-status-notifier-host) and is currently running. If no StatusNotifierHost are registered and running, all `StatusNotifierItem` instances should fall back using the Freedesktop System tray specification.

#### 4.1.3 ProtocolVersion [#](#status-notifier-watcher-properties-protocol-version)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-properties-protocol-version

`INT org.freedesktop.StatusNotifierWatcher.ProtocolVersion ();`

The version of the protocol the StatusNotifierWatcher instance implements.

### 4.2 Methods [#](#status-notifier-watcher-methods)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-methods

#### 4.2.1 RegisterStatusNotifierItem [#](#status-notifier-watcher-methods-register-status-notifier-item)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-methods-register-status-notifier-item

`VOID org.freedesktop.StatusNotifierWatcher.RegisterStatusNotifierItem (STRING service);`

Register a StatusNotifierItem into the StatusNotifierWatcher, in the form of its full name on the session bus, for instance org.freedesktop.StatusNotifierItem-4077-1. A StatusNotifierItem instance must be registered to the watcher in order to be noticed from both the watcher and the StatusNotifierHost instances. If the registered StatusNotifierItem goes away from the session bus, the StatusNotifierWatcher should automatically notice it and remove it from the list of registered services.

#### 4.2.2 RegisterStatusNotifierHost [#](#status-notifier-watcher-methods-register-status-notifier-host)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-methods-register-status-notifier-host

`VOID org.freedesktop.StatusNotifierWatcher.RegisterStatusNotifierHost (STRING service);`

Register a StatusNotifierHost into the StatusNotifierWatcher, in the form of its full name on the session bus, for instance org.freedesktop.StatusNotifierHost-4005. Every NotficationHost instance that intends to display StatusNotifierItem representations should register to StatusNotifierWatcher with this method. The StatusNotifierWatcher should automatically notice if an instance of StatusNotifierHost goes away.

### 4.3 Signals [#](#status-notifier-watcher-signals)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-signals

#### 4.3.1 StatusNotifierItemRegistered [#](#status-notifier-watcher-signals-status-notifier-item-registered)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-signals-status-notifier-item-registered

`BOOL org.freedesktop.StatusNotifierWatcher.StatusNotifierItemRegistered (STRING service);`

A new StatusNotifierItem has been registered, the argument of the signal is the session bus name of the instance. StatusNotifierHost implementation should listen this signal to know when they should update their representation of the items.

#### 4.3.2 StatusNotifierItemUnregistered [#](#status-notifier-watcher-signals-status-notifier-item-unregistered)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-signals-status-notifier-item-unregistered

`BOOL org.freedesktop.StatusNotifierWatcher.StatusNotifierItemUnregistered (STRING service);`

A StatusNotifierItem instance has disappeared from the bus, the argument of the signal is the session bus name of the instance. StatusNotifierHost implementation should listen this signal to know when they should update their representation of the items.

#### 4.3.3 StatusNotifierHostRegistered [#](#status-notifier-watcher-signals-status-notifier-host-registered)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-watcher-signals-status-notifier-host-registered

`BOOL org.freedesktop.StatusNotifierWatcher.StatusNotifierHostRegistered ();`

A new StatusNotifierHost has been registered, the StatusNotifierItem instances knows that they can use this protocol instead of the Freedesktop System tray protocol.

## 5 StatusNotifierHost [#](#status-notifier-host)

- File Name: status-notifier-item-spec.xml
- ID: status-notifier-host

 StatusNotifierHost corresponds an object in an application that does the actual graphical representation of StatusNotifierItem instances.

 Instances of this service are registered on the D-Bus session bus, under a name on the form org.freedesktop.StatusNotifierHost-id where id is an unique identifier, that keeps the names unique on the bus, such as the process-id of the application or another type of identifier if more that one StatusNotifierHost is registered by the same process.

 StatusNotifierHost instances don't need any particular property, method or signal registered to the bus. What is important is the presence of the service on the bus, to inform StatusNotifierWatcher that a StatusNotifierItem representation is currently running on the system.

## 6 Icon pixmap [#](#icon-pixmap)

- File Name: status-notifier-item-spec.xml
- ID: icon-pixmap

 Icon pixmaps can be transferred over the bus by a particular serialization of their data, capable of representing multiple resolutions of the same image or a brief animation of images of the same size.

 Icon pixmaps are transferred in an array of raw image data structures of signature `a(iiay)`, with each one describing the width, height, and image data respectively. The data is represented in `ARGB32` format and is in the network byte order, to make easy the communication over the network between little and big endian machines.

## 7 Markup [#](#markup)

- File Name: status-notifier-item-spec.xml
- ID: markup

The sub text of a tooltip may contain markup. The markup is XML-based, and consists of a small subset of XHTML.

The following tags should be supported by the StatusNotifierHost visualization. Though it is optional, it is recommended. StatusNotifierHost that do not support these tags should filter them out.

`<b> ... </b>` Bold

`<i> ... </i>` Italic

`<u> ... </u>` Underline

`<a href="..."> ... </a>` Hyperlink

`<img src="..." alt="..."/>` Image

Images can be used with absolute pathnames as image source. The alt text is mandatory.

A full-blown HTML implementation is not required of this spec, and notifications should never take advantage of tags that are not listed above. As tooltips are not a substitute for complex dialogs, advanced layout is not necessary, and may in fact limit the number of systems where this system can run on, due to memory usage and screen space. Such examples are PDAs, certain cell phones, and slow PCs or laptops with little memory.

For the same reason, a full XML or XHTML implementation using XSLT or CSS stylesheets is not part of this specification. Information that must be presented in a more complex form should use an application-specific dialog, a web browser, or some other display mechanism.

The tags specified above mark up the content in a way that allows them to be stripped out on some implementations without impacting the actual content.

© 2026 Freedesktop.org
