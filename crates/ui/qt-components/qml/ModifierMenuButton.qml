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
    SearchMenu {
        id: menu
        width: 280
        labels: root.labels
        placeholderText: qsTr("Search modifiers")
        onActivated: function(index) {
            const value = backend.valueAt(root.values, index)
            if (value.length > 0)
                root.selected(value)
        }
    }
}
