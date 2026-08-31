import QtQuick
import dev.shrimply.components
import QtQuick.Controls

ScrollView {
    id: root
    property string value: ""
    property int minimumContentHeight: 96
    property int maximumLength: 0
    property int activeTypo: -1
    signal edited(string value)
    signal committed(string value)
    implicitHeight: minimumContentHeight

    TextInputBackend {
        id: backend
        onChanged: function(value) {
            root.value = value
            root.edited(value)
            commitTimer.restart()
        }
        onCommitted: function(value) { root.committed(value) }
    }
    Timer { id: commitTimer; interval: 750; onTriggered: backend.commit() }

    Component.onCompleted: {
        backend.configure(value, maximumLength)
        editor.text = backend.text
    }
    Component.onDestruction: backend.commit()
    onValueChanged: if (!editor.activeFocus && value !== backend.text) {
        backend.configure(value, maximumLength)
        editor.text = backend.text
    }

    TextArea {
        id: editor
        wrapMode: TextEdit.WrapAtWordBoundaryOrAnywhere
        selectByMouse: true
        persistentSelection: true
        padding: 8
        TypoHighlighter { document: editor.textDocument; ranges: backend.typoRanges }
        Repeater {
            model: backend.typoCount
            TypoUnderline {
                required property int index
                editor: editor
                typoBackend: backend
                typoIndex: index
            }
        }
        ToolTip.visible: root.activeTypo >= 0 && typoHover.hovered
        ToolTip.text: backend.typoMessage(root.activeTypo)
        HoverHandler {
            id: typoHover
            onPointChanged: root.activeTypo = backend.typoAt(
                editor.positionAt(point.position.x, point.position.y))
        }
        onTextChanged: if (activeFocus) {
            const accepted = backend.edit(text)
            if (accepted !== text)
                text = accepted
        }
        onActiveFocusChanged: if (!activeFocus) {
            commitTimer.stop()
            backend.commit()
        }

        TapHandler {
            acceptedButtons: Qt.RightButton
            onTapped: function(eventPoint) {
                root.activeTypo = backend.typoAt(
                    editor.positionAt(eventPoint.position.x, eventPoint.position.y))
                if (root.activeTypo >= 0)
                    typoMenu.popup()
            }
        }
        Menu {
            id: typoMenu
            Instantiator {
                model: 6
                delegate: MenuItem {
                    required property int index
                    text: backend.typoCorrection(root.activeTypo, index)
                    visible: text.length > 0
                    onTriggered: editor.text = backend.applyCorrection(root.activeTypo, index)
                }
                onObjectAdded: function(index, object) { typoMenu.insertItem(index, object) }
                onObjectRemoved: function(index, object) { typoMenu.removeItem(object) }
            }
        }
    }
}
