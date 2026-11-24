import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: root
    width: 393; height: 820
    visible: true
    title: "Spectre Camera"
    color: "#0b1010"
    font.family: "Sans Serif"
    property color accent: "#c6f6be"
    property bool showGrid: false
    property int delaySeconds: 0
    property int countdown: 0
    property bool reviewing: false
    property bool showExposure: false
    property bool showInfo: false
    property bool lossless: false

    component ControlButton: AbstractButton {
        id: control
        property string glyph: ""
        property bool selected: false
        implicitWidth: 48; implicitHeight: 48
        Accessible.name: text
        background: Rectangle {
            radius: width / 2
            color: control.down ? "#496153" : control.selected ? "#2b4036" : "#1a2322"
            border.width: control.activeFocus ? 2 : 0
            border.color: root.accent
        }
        contentItem: Text {
            text: control.glyph; color: control.selected ? root.accent : "#edf3ef"
            font.pixelSize: 23; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter
        }
    }

    ColumnLayout {
        anchors.fill: parent; spacing: 0
        Item {
            Layout.fillWidth: true; Layout.preferredHeight: 88
            RowLayout {
                anchors.fill: parent; anchors.leftMargin: 22; anchors.rightMargin: 22; spacing: 10
                Column {
                    Layout.fillWidth: true; spacing: 5
                    Text { text: "SPECTRE"; color: root.accent; font.pixelSize: 12; font.letterSpacing: 3 }
                    Text { text: "Camera"; color: "#f4f6f1"; font.pixelSize: 27; font.weight: Font.Medium }
                }
                ControlButton { glyph: "↻"; text: "Rotate image 90 degrees"; enabled: !camera.busy; onClicked: camera.rotateImage() }
                ControlButton { glyph: root.showGrid ? "▦" : "⊞"; text: "Composition grid"; selected: root.showGrid; onClicked: root.showGrid = !root.showGrid }
                ControlButton { glyph: root.delaySeconds ? "3s" : "◷"; text: "Three second timer"; selected: root.delaySeconds > 0; onClicked: root.delaySeconds = root.delaySeconds ? 0 : 3 }
                ControlButton { glyph: "i"; text: "Camera information"; selected: root.showInfo; onClicked: root.showInfo = !root.showInfo }
            }
        }
        Rectangle {
            id: viewfinder
            Layout.fillWidth: true; Layout.fillHeight: true
            color: "#141c1b"; clip: true
            Image {
                id: liveImage; objectName: "livePreview"
                anchors.centerIn: parent; source: camera.preview; cache: false
                readonly property bool sideways: camera.previewRotation % 180 !== 0
                width: sideways ? parent.height : parent.width
                height: sideways ? parent.width : parent.height
                rotation: camera.previewRotation
                fillMode: Image.PreserveAspectFit
            }
            Item {
                width: liveImage.sideways ? liveImage.paintedHeight : liveImage.paintedWidth
                height: liveImage.sideways ? liveImage.paintedWidth : liveImage.paintedHeight
                anchors.centerIn: parent; visible: root.showGrid && camera.ready
                Repeater {
                    model: 2
                    Rectangle { required property int index; x: parent.width * (index + 1) / 3; width: 1; height: parent.height; color: "#75ffffff" }
                }
                Repeater {
                    model: 2
                    Rectangle { required property int index; y: parent.height * (index + 1) / 3; height: 1; width: parent.width; color: "#75ffffff" }
                }
            }
            Rectangle {
                anchors.top: parent.top; anchors.horizontalCenter: parent.horizontalCenter; anchors.topMargin: 16
                width: badge.implicitWidth + 24; height: 29; radius: 15; color: "#b50b1010"
                Text { id: badge; anchors.centerIn: parent; text: demoMode ? "DEMO PREVIEW" : "LIBCAMERA  ·  REAR"; color: root.accent; font.pixelSize: 10; font.letterSpacing: 1.4 }
            }
            Column {
                anchors.centerIn: parent; width: parent.width - 56; spacing: 18
                visible: !camera.ready
                Text { width: parent.width; text: "◉"; color: root.accent; font.pixelSize: 48; horizontalAlignment: Text.AlignHCenter }
                Text { width: parent.width; text: camera.message; color: "#d5ded8"; font.pixelSize: 15; wrapMode: Text.WordWrap; horizontalAlignment: Text.AlignHCenter }
                Button { anchors.horizontalCenter: parent.horizontalCenter; text: "Try again"; visible: !camera.busy; onClicked: camera.start() }
            }
            Text {
                anchors.centerIn: parent; visible: root.countdown > 0
                text: root.countdown; font.pixelSize: 112; font.weight: Font.Light; color: "white"
            }
            Rectangle {
                id: flash; anchors.fill: parent; color: "white"; opacity: 0
                SequentialAnimation { id: flashAnimation; NumberAnimation { target: flash; property: "opacity"; to: 0.65; duration: 60 } NumberAnimation { target: flash; property: "opacity"; to: 0; duration: 230 } }
            }
            Rectangle {
                anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter; anchors.bottomMargin: 16
                width: 280; height: 74; radius: 18; color: "#e61a2322"; visible: root.showExposure
                Column {
                    anchors.fill: parent; anchors.margins: 10; spacing: 2
                    Text { text: camera.exposureSupported ? "EXPOSURE  " + brightness.value.toFixed(1) + " EV" : "AUTO EXPOSURE · WHITE BALANCE"; color: root.accent; font.pixelSize: 10; font.letterSpacing: 1 }
                    Slider { id: brightness; width: parent.width; height: 34; from: -2; to: 3; value: 0; stepSize: 0.1; enabled: camera.exposureSupported; Accessible.name: "Sensor exposure compensation"; onValueChanged: camera.setBrightness(value) }
                }
            }
        }
        Item {
            Layout.fillWidth: true; Layout.preferredHeight: 184
            ColumnLayout {
                anchors.fill: parent; anchors.margins: 16; spacing: 12
                Text { Layout.alignment: Qt.AlignHCenter; text: root.lossless ? "PHOTO · LOSSLESS PNG" : "PHOTO · JPEG"; color: root.accent; font.pixelSize: 11; font.letterSpacing: 2.5 }
                RowLayout {
                    Layout.fillWidth: true; spacing: 22
                    Item { Layout.fillWidth: true }
                    AbstractButton {
                        implicitWidth: 54; implicitHeight: 62; enabled: camera.photo !== ""
                        Accessible.name: "View last photo"
                        onClicked: root.reviewing = true
                        contentItem: Rectangle {
                            radius: 13; color: "#1a2322"; clip: true
                            Image { anchors.fill: parent; anchors.margins: 3; source: camera.photo; asynchronous: true; sourceSize: Qt.size(192,192); fillMode: Image.PreserveAspectCrop }
                            Text { anchors.centerIn: parent; visible: camera.photo === ""; text: "▧"; font.pixelSize: 25; color: "#82928b" }
                        }
                    }
                    AbstractButton {
                        id: shutter
                        implicitWidth: 82; implicitHeight: 82
                        enabled: camera.ready && !camera.busy && root.countdown === 0
                        Accessible.name: "Take photo"
                        onClicked: {
                            if (root.delaySeconds) { root.countdown = root.delaySeconds; countdownTimer.start() }
                            else camera.capture()
                        }
                        background: Rectangle { radius: 41; color: "transparent"; border.color: shutter.enabled ? root.accent : "#4a5851"; border.width: 2 }
                        contentItem: Item { Rectangle { anchors.centerIn: parent; width: 68; height: 68; radius: 34; color: shutter.down ? "#91c98b" : shutter.enabled ? root.accent : "#4a5851" } }
                    }
                    ControlButton { glyph: "☼"; text: "Exposure information"; selected: root.showExposure; onClicked: root.showExposure = !root.showExposure }
                    Item { Layout.fillWidth: true }
                }
                Text {
                    Layout.fillWidth: true; Layout.preferredHeight: 22
                    text: camera.message; color: "#91a198"; font.pixelSize: 11
                    horizontalAlignment: Text.AlignHCenter; elide: Text.ElideRight
                }
            }
        }
    }
    Timer {
        id: countdownTimer; interval: 1000; repeat: true
        onTriggered: { root.countdown--; if (!root.countdown) { stop(); if (camera.ready) camera.capture() } }
    }
    Connections {
        target: camera
        function onCaptured() { flashAnimation.start() }
        function onChanged() { if (!camera.ready) { countdownTimer.stop(); root.countdown = 0 } }
    }
    Rectangle {
        anchors.fill: parent; visible: root.reviewing; color: "#0b1010"; z: 10
        Image { anchors.fill: parent; anchors.margins: 18; source: root.reviewing ? camera.photo : ""; asynchronous: true; sourceSize: Qt.size(1200,1600); fillMode: Image.PreserveAspectFit }
        ControlButton { anchors.top: parent.top; anchors.left: parent.left; anchors.margins: 20; glyph: "‹"; text: "Back to camera"; onClicked: root.reviewing = false }
        Text { anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter; anchors.bottomMargin: 30; text: "Saved on this phone"; color: root.accent; font.pixelSize: 12 }
    }
    Popup {
        anchors.centerIn: parent; width: Math.min(root.width - 32, 360); modal: true
        visible: root.showInfo
        onClosed: root.showInfo = false
        padding: 22
        background: Rectangle { color: "#202e28"; radius: 24 }
        contentItem: Column {
            spacing: 16
            Text { text: "Made for your Poco F1"; color: root.accent; font.pixelSize: 20 }
            Text {
                width: parent.width; wrapMode: Text.WordWrap; color: "#d2ded4"; font.pixelSize: 14
                text: "Sony IMX363 · main rear camera\nFull-resolution libcamera processing\nAuto exposure and white balance\n\nSensor colour calibration is pending. Autofocus, flash, video and the front camera are not available. Exposure compensation is enabled only when supported by the pipeline."
            }
            Switch { text: "Save lossless PNG"; checked: root.lossless; onToggled: { root.lossless = checked; camera.setLossless(checked) } }
            Button { text: "Got it"; onClicked: root.showInfo = false }
        }
    }
    Shortcut { sequence: "Escape"; onActivated: { root.reviewing = false; root.showInfo = false; countdownTimer.stop(); root.countdown = 0 } }
    Shortcut { sequence: "Space"; onActivated: { if (shutter.enabled) shutter.clicked() } }
}
