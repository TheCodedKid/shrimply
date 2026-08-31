import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: root
    property alias value: editor.value
    property alias output: result.text
    signal edited(string value)
    spacing: 6

    CodeEditor {
        id: editor
        Layout.fillWidth: true
        Layout.preferredHeight: 180
        onEdited: function(value) { root.edited(value) }
    }
    ReadOnlyField {
        id: result
        Layout.fillWidth: true
    }
}
