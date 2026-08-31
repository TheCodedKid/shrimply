pragma ComponentBehavior: Bound

import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

Button {
    id: root
    property var values: []
    property var labels: []
    signal selected(string value)
    text: qsTr("Add modifier")
    flat: true
    icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/add-modifier.svg"
    icon.color: palette.buttonText
    display: AbstractButton.TextBesideIcon
    onClicked: menu.popup(anchor, 0, 0)

    SelectorBackend { id: backend }
    Item {
        id: anchor
        parent: root
        x: (root.width - menu.width) / 2
        y: root.height
        width: 1
        height: 1
        visible: false
    }
    Menu {
        id: menu
        width: 280
        padding: 6
        focus: true
        popupType: Popup.Window
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent
        onOpened: {
            search.clear()
            search.forceActiveFocus(Qt.PopupFocusReason)
        }
        contentItem: ColumnLayout {
            spacing: 6
            TextField {
                id: search
                Layout.fillWidth: true
                placeholderText: qsTr("Search modifiers")
                selectByMouse: true
                Keys.onEscapePressed: function(event) {
                    menu.close()
                    event.accepted = true
                }
            }
            ListView {
                id: choices
                Layout.fillWidth: true
                Layout.preferredHeight: Math.min(contentHeight, 280)
                clip: true
                model: root.labels
                delegate: ItemDelegate {
                    required property int index
                    width: choices.width
                    text: root.labels[index]
                    visible: backend.matchesQuery(text, search.text)
                    height: visible ? implicitHeight : 0
                    onClicked: {
                        const value = backend.valueAt(root.values, index)
                        if (value.length > 0)
                            root.selected(value)
                        menu.close()
                    }
                }
            }
        }
    }
}
