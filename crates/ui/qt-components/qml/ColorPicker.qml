import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import QtQuick.Window

Item {
    id: root
    property color color: "#000000"
    property string title: "Select color"
    property bool withAlpha: true
    signal selected(color color)
    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight

    SystemPalette { id: systemPalette }

    ColorPickerBackend {
        id: backend
        onSelected: function(value) {
            root.color = value
            root.selected(value)
        }
        onConfirmed: picker.close()
    }
    Component.onCompleted: backend.configure(root.color, root.withAlpha)
    onColorChanged: backend.configure(root.color, root.withAlpha)
    onWithAlphaChanged: backend.configure(root.color, root.withAlpha)

    Button {
        id: button
        anchors.fill: parent
        onClicked: {
            backend.configure(root.color, root.withAlpha)
            picker.show()
            picker.requestActivate()
        }
        contentItem: Item {
            implicitWidth: buttonContent.implicitWidth
            implicitHeight: buttonContent.implicitHeight
            RowLayout {
                id: buttonContent
                anchors.centerIn: parent
                spacing: 8
                Rectangle {
                    implicitWidth: 22
                    implicitHeight: 22
                    radius: 4
                    color: backend.color
                    border.color: systemPalette.mid
                }
                Label { text: backend.hex; font.family: "monospace" }
            }
        }
    }

    Window {
        id: picker
        width: 900
        height: 560
        visible: false
        flags: Qt.Dialog
        modality: Qt.NonModal
        transientParent: root.Window.window
        title: ComponentTranslations.text(root.title)
        color: systemPalette.window
        onVisibleChanged: if (visible) backend.configure(root.color, root.withAlpha)

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 12

            RowLayout {
                Layout.fillWidth: true
                ToolButton {
                    icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/screen-color.svg"
                    icon.color: palette.buttonText
                    display: AbstractButton.IconOnly
                    enabled: !backend.screenPicking
                    ToolTip.visible: hovered
                    ToolTip.text: ComponentTranslations.text("Pick screen color")
                    onClicked: backend.pickScreenColor()
                }
                Button {
                    flat: true
                    implicitWidth: 42
                    implicitHeight: 36
                    padding: 4
                    ToolTip.visible: hovered
                    ToolTip.text: ComponentTranslations.text("Open the system color picker")
                    onClicked: nativeDialog.open()
                    contentItem: Rectangle {
                        radius: 4
                        color: backend.draft
                        border.color: systemPalette.mid
                    }
                }
                TextField {
                    id: hex
                    Layout.preferredWidth: 280
                    text: backend.hex
                    font.family: "monospace"
                    onAccepted: if (!backend.applyHex(text)) selectAll()
                    onActiveFocusChanged: if (!activeFocus && !backend.applyHex(text))
                        text = backend.hex
                }
                Item { Layout.fillWidth: true }
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 20

                ColumnLayout {
                    Layout.alignment: Qt.AlignTop
                    spacing: 8

                    RowLayout {
                        spacing: 8
                        Slider {
                            id: hue
                            Layout.preferredHeight: colorPlane.height
                            orientation: Qt.Vertical
                            from: 360
                            to: 0
                            value: backend.hue
                            ToolTip.visible: hovered
                            ToolTip.text: ComponentTranslations.text("Hue")
                            onMoved: backend.setHsva(
                                value, backend.saturation, backend.brightness, backend.alpha)
                        }

                        Rectangle {
                            id: colorPlane
                            Layout.preferredWidth: 300
                            Layout.preferredHeight: 300
                            color: Qt.hsva(backend.hue / 360, 1, 1, 1)
                            Rectangle {
                                anchors.fill: parent
                                gradient: Gradient {
                                    orientation: Gradient.Horizontal
                                    GradientStop { position: 0; color: "white" }
                                    GradientStop { position: 1; color: "transparent" }
                                }
                            }
                            Rectangle {
                                anchors.fill: parent
                                gradient: Gradient {
                                    GradientStop { position: 0; color: "transparent" }
                                    GradientStop { position: 1; color: "black" }
                                }
                            }
                            Rectangle {
                                x: backend.saturation * parent.width - width / 2
                                y: (1 - backend.brightness) * parent.height - height / 2
                                width: 14
                                height: 14
                                radius: 7
                                color: "transparent"
                                border.width: 2
                                border.color: "white"
                            }
                            MouseArea {
                                anchors.fill: parent
                                onPressed: function(mouse) { update(mouse) }
                                onPositionChanged: function(mouse) { if (pressed) update(mouse) }
                                function update(mouse) {
                                    backend.setHsva(backend.hue,
                                        Math.max(0, Math.min(1, mouse.x / width)),
                                        1 - Math.max(0, Math.min(1, mouse.y / height)),
                                        backend.alpha)
                                }
                            }
                        }
                    }

                    RowLayout {
                        visible: root.withAlpha
                        Item { Layout.preferredWidth: hue.width }
                        Label { text: ComponentTranslations.text("Alpha") }
                        Slider {
                            id: alpha
                            Layout.fillWidth: true
                            from: 0
                            to: 1
                            value: backend.alpha
                            background: Item {
                                x: alpha.leftPadding
                                y: alpha.topPadding + alpha.availableHeight / 2 - height / 2
                                implicitWidth: 200
                                implicitHeight: 10
                                width: alpha.availableWidth
                                height: implicitHeight
                                clip: true
                                Checkerboard { anchors.fill: parent }
                                Rectangle {
                                    anchors.fill: parent
                                    gradient: Gradient {
                                        orientation: Gradient.Horizontal
                                        GradientStop {
                                            position: 0
                                            color: Qt.rgba(
                                                backend.draft.r,
                                                backend.draft.g,
                                                backend.draft.b,
                                                0)
                                        }
                                        GradientStop {
                                            position: 1
                                            color: Qt.rgba(
                                                backend.draft.r,
                                                backend.draft.g,
                                                backend.draft.b,
                                                1)
                                        }
                                    }
                                }
                            }
                            onMoved: backend.setHsva(
                                backend.hue, backend.saturation, backend.brightness, value)
                        }
                    }

                }

                Rectangle {
                    Layout.fillHeight: true
                    Layout.preferredWidth: 1
                    color: systemPalette.mid
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Label { text: ComponentTranslations.text("Palette") }
                    GridLayout {
                        columns: 9
                        Repeater {
                            model: backend.paletteCount
                            Rectangle {
                                required property int index
                                implicitWidth: 38
                                implicitHeight: 32
                                radius: 4
                                color: backend.paletteColor(index)
                                border.width: color === backend.draft ? 3 : 1
                                border.color: color === backend.draft
                                    ? systemPalette.highlight : systemPalette.mid
                                ToolTip.visible: swatchHover.hovered
                                ToolTip.text: backend.paletteLabel(index)
                                HoverHandler { id: swatchHover }
                                TapHandler { onTapped: backend.chooseColor(parent.color) }
                            }
                        }
                    }
                    Label {
                        visible: backend.recentCount > 0
                        text: ComponentTranslations.text("Recent")
                    }
                    RowLayout {
                        Repeater {
                            model: backend.recentCount
                            Rectangle {
                                required property int index
                                implicitWidth: 40
                                implicitHeight: 30
                                radius: 4
                                color: backend.recentColor(index)
                                border.color: systemPalette.mid
                                TapHandler { onTapped: backend.chooseColor(parent.color) }
                            }
                        }
                    }
                    Item { Layout.fillHeight: true }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Item { Layout.fillWidth: true }
                Button {
                    text: ComponentTranslations.text("Cancel")
                    onClicked: picker.close()
                }
                Button {
                    text: ComponentTranslations.text("Select")
                    highlighted: true
                    onClicked: backend.confirm()
                }
            }
        }
    }

    ColorDialog {
        id: nativeDialog
        title: ComponentTranslations.text(root.title)
        selectedColor: backend.draft
        options: root.withAlpha ? ColorDialog.ShowAlphaChannel : ColorDialog.NoOption
        onAccepted: backend.chooseColor(selectedColor)
    }
}
