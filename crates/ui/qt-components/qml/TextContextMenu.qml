import QtQuick
import QtQuick.Controls

Menu {
    id: root
    required property var editor
    required property var typoBackend
    property int typoIndex: -1
    readonly property int correctionCount: typoBackend.typoCorrectionCount(typoIndex)
    popupType: Popup.Window

    function openAt(x, y, typo) {
        typoIndex = typo
        popup(editor, x, y)
    }

    onClosed: typoIndex = -1

    Instantiator {
        model: root.correctionCount
        delegate: MenuItem {
            required property int index
            text: root.typoBackend.typoCorrection(root.typoIndex, index)
            onTriggered: {
                root.editor.text = root.typoBackend.applyCorrection(root.typoIndex, index)
                root.editor.forceActiveFocus(Qt.PopupFocusReason)
            }
        }
        onObjectAdded: function(index, object) { root.insertItem(index, object) }
        onObjectRemoved: function(index, object) { root.removeItem(object) }
    }

    Instantiator {
        model: root.correctionCount > 0 ? 1 : 0
        delegate: MenuSeparator {}
        onObjectAdded: function(index, object) { root.insertItem(root.correctionCount, object) }
        onObjectRemoved: function(index, object) { root.removeItem(object) }
    }

    MenuItem {
        text: qsTr("Undo")
        enabled: root.editor.canUndo
        onTriggered: root.editor.undo()
    }
    MenuItem {
        text: qsTr("Redo")
        enabled: root.editor.canRedo
        onTriggered: root.editor.redo()
    }
    MenuSeparator {}
    MenuItem {
        text: qsTr("Cut")
        enabled: root.editor.selectedText.length > 0
        onTriggered: root.editor.cut()
    }
    MenuItem {
        text: qsTr("Copy")
        enabled: root.editor.selectedText.length > 0
        onTriggered: root.editor.copy()
    }
    MenuItem {
        text: qsTr("Paste")
        enabled: root.editor.canPaste
        onTriggered: root.editor.paste()
    }
    MenuItem {
        text: qsTr("Delete")
        enabled: root.editor.selectedText.length > 0
        onTriggered: root.editor.remove(root.editor.selectionStart, root.editor.selectionEnd)
    }
    MenuSeparator {}
    MenuItem {
        text: qsTr("Select All")
        enabled: root.editor.length > 0
        onTriggered: root.editor.selectAll()
    }
}
