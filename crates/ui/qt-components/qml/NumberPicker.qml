import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    property real value: 0
    property real minimum: -1000000
    property real maximum: 1000000
    property real dragStep: 1
    property real dragPixels: 3
    property int digits: 2
    property string prefix: ""
    property string prefixIconName: ""
    property url prefixIconSource
    property bool prefixIconRotates: false
    property real prefixIconRotationOffsetDegrees: 0
    property string suffix: ""
    property string unitName: ""
    property int widthCharacters: 8
    signal edited(real value)
    signal committed(real value)

    implicitWidth: widthCharacters * 12
    implicitHeight: Math.max(displayButton.implicitHeight, editor.implicitHeight)

    function configureBackend() {
        backend.configure(value, minimum, maximum, dragStep, dragPixels, digits)
    }

    NumberInputBackend {
        id: backend
        onEdited: function(next) {
            root.edited(next)
        }
        onCommitted: function(next) { root.committed(next) }
    }

    Component.onCompleted: configureBackend()
    onMinimumChanged: configureBackend()
    onMaximumChanged: configureBackend()
    onDragStepChanged: configureBackend()
    onDragPixelsChanged: configureBackend()
    onDigitsChanged: configureBackend()
    onValueChanged: backend.setExternalValue(value)

    Button {
        id: displayButton
        anchors.fill: parent
        visible: !backend.editing
        hoverEnabled: true

        contentItem: RowLayout {
            spacing: 4
            Loader {
                active: root.prefixIconSource.toString().length > 0 || root.prefixIconName.length > 0
                Layout.preferredWidth: 16
                Layout.preferredHeight: 16
                sourceComponent: ToolButton {
                    icon.source: root.prefixIconSource.toString().length > 0
                        ? root.prefixIconSource
                        : "image://theme/" + root.prefixIconName
                    icon.color: palette.buttonText
                    display: AbstractButton.IconOnly
                    enabled: false
                    opacity: 1
                    padding: 0
                    background: null
                    rotation: root.prefixIconRotates
                        ? root.value + root.prefixIconRotationOffsetDegrees
                        : 0
                }
            }
            Label { visible: root.prefix.length > 0; text: root.prefix; opacity: 0.72 }
            Label {
                text: backend.displayText
                horizontalAlignment: Text.AlignRight
                Layout.fillWidth: true
                font.family: "monospace"
            }
            Label {
                visible: root.suffix.length > 0 || root.unitName.length > 0
                text: root.suffix.length > 0 ? root.suffix : root.unitName
                opacity: 0.72
            }
        }

        DragInput {
            id: dragArea
            anchors.fill: parent
            threshold: backend.dragThreshold
            onDragStarted: backend.beginDrag()
            onDragged: function(offset) { backend.drag(offset) }
            onDragFinished: backend.endDrag()
            onClicked: {
                backend.beginEdit()
                editor.text = backend.displayText
                editor.forceActiveFocus()
                editor.selectAll()
            }
        }
    }

    TextField {
        id: editor
        anchors.fill: parent
        visible: backend.editing
        inputMethodHints: Qt.ImhFormattedNumbersOnly
        horizontalAlignment: TextInput.AlignRight
        onTextEdited: backend.previewText(text)
        onAccepted: backend.commitText(text)
        onActiveFocusChanged: if (!activeFocus && backend.editing) backend.commitText(text)
    }
}
