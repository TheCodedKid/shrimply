import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: root
    property alias value: editor.value
    property alias output: result.text
    property string error: ""
    signal edited(string value)
    signal committed(string value)
    spacing: 6

    CodeEditor {
        id: editor
        Layout.fillWidth: true
        Layout.preferredHeight: 180
        onEdited: function(value) { root.edited(value) }
        onCommitted: function(value) { root.committed(value) }
    }
    ReadOnlyField {
        id: result
        Layout.fillWidth: true
        ToolTip.visible: hovered && root.error.length > 0
        ToolTip.text: root.error
    }
}
