import QtQuick
import QtQuick.Controls
import dev.shrimply.components.native 1.0

ScrollView {
    id: root
    property string value: ""
    property int tabWidth: 4
    signal edited(string value)
    signal committed(string value)
    implicitHeight: 180

    Component.onDestruction: root.committed(editor.text)

    onValueChanged: if (editor.text !== value) editor.text = value

    TextArea {
        id: editor
        text: root.value
        wrapMode: TextEdit.NoWrap
        selectByMouse: true
        leftPadding: 48
        font.family: "monospace"
        onTextChanged: if (root.value !== text) {
            root.edited(text)
        }
        onActiveFocusChanged: if (!activeFocus) root.committed(text)
        Keys.onTabPressed: function(event) {
            const spaces = " ".repeat(root.tabWidth)
            insert(cursorPosition, spaces)
            event.accepted = true
        }

        CodeHighlighter {
            document: editor.textDocument
        }

        Rectangle {
            x: 0
            y: 0
            width: 40
            height: editor.contentHeight + editor.topPadding + editor.bottomPadding
            color: editor.palette.alternateBase
            z: -1
        }
        Repeater {
            model: editor.lineCount
            Label {
                required property int index
                x: 4
                y: editor.topPadding + index * implicitHeight
                width: 32
                horizontalAlignment: Text.AlignRight
                text: index + 1
                color: editor.palette.placeholderText
                font: editor.font
            }
        }
    }
}
