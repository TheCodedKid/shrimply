pragma ComponentBehavior: Bound

import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

Button {
    id: root
    property var values: []
    property var labels: []
    property string value: ""
    property bool enableSearch: backend.searchable(labels.length)
    property string searchPlaceholder: ComponentTranslations.text("Search")
    signal selected(string value)
    text: currentLabel()

    function currentLabel() {
        const index = backend.selectedIndex(values, value)
        return index >= 0 && index < labels.length ? labels[index] : ""
    }

    function synchronize() {
        choices.currentIndex = backend.selectedIndex(values, value)
    }

    function choose(index) {
        const next = backend.valueAt(values, index)
        if (index < 0 || next.length === 0)
            return
        value = next
        selected(next)
        popup.close()
    }

    function moveSelection(direction) {
        const next = backend.nextMatchingIndex(
            labels, search.text, choices.currentIndex, direction)
        if (next >= 0) {
            choices.currentIndex = next
            choices.positionViewAtIndex(next, ListView.Contain)
        }
    }

    SelectorBackend { id: backend }

    // Match the editor timeline menus: QQuickMenu's Wayland positioner uses
    // the parent item's bottom-left anchor with bottom-right gravity.  A wide
    // parent therefore places the popup at the parent's right edge; using a
    // one-pixel anchor makes that edge the field's left edge.
    Item {
        id: popupAnchor
        parent: root
        x: 0
        y: root.height
        width: 1
        height: 1
        visible: false
    }

    Component.onCompleted: synchronize()
    onValuesChanged: synchronize()
    onValueChanged: synchronize()
    onClicked: {
        if (popup.opened)
            popup.close()
        else
            popup.popup(popupAnchor, 0, 0)
    }

    contentItem: RowLayout {
        Label {
            text: root.text
            elide: Text.ElideRight
            Layout.fillWidth: true
        }
        Label { text: "⌄" }
    }

    Menu {
        id: popup
        width: Math.max(root.width, 240)
        padding: 6
        focus: true
        popupType: Popup.Window
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent
        onOpened: {
            search.clear()
            choices.currentIndex = backend.selectedIndex(root.values, root.value)
            if (root.enableSearch)
                search.forceActiveFocus(Qt.PopupFocusReason)
            else
                choices.forceActiveFocus(Qt.PopupFocusReason)
        }
        onClosed: root.forceActiveFocus(Qt.PopupFocusReason)

        contentItem: ColumnLayout {
            spacing: 6

            TextField {
                id: search
                visible: root.enableSearch
                Layout.fillWidth: true
                placeholderText: root.searchPlaceholder
                selectByMouse: true
                onTextChanged: choices.currentIndex = backend.nextMatchingIndex(
                    root.labels, text, -1, 1)
                Keys.onDownPressed: function(event) {
                    root.moveSelection(1)
                    choices.forceActiveFocus(Qt.TabFocusReason)
                    event.accepted = true
                }
                Keys.onUpPressed: function(event) {
                    root.moveSelection(-1)
                    choices.forceActiveFocus(Qt.TabFocusReason)
                    event.accepted = true
                }
                Keys.onEscapePressed: function(event) {
                    popup.close()
                    event.accepted = true
                }
                onAccepted: root.choose(choices.currentIndex)
            }

            ListView {
                id: choices
                Layout.fillWidth: true
                Layout.preferredHeight: Math.min(contentHeight, 280)
                clip: true
                model: root.labels
                keyNavigationEnabled: false
                delegate: ItemDelegate {
                    required property int index
                    width: choices.width
                    text: root.labels[index]
                    visible: backend.matchesQuery(text, search.text)
                    height: visible ? implicitHeight : 0
                    highlighted: choices.currentIndex === index
                    onHoveredChanged: if (hovered) choices.currentIndex = index
                    onClicked: root.choose(index)
                }
                Keys.onDownPressed: function(event) {
                    root.moveSelection(1)
                    event.accepted = true
                }
                Keys.onUpPressed: function(event) {
                    root.moveSelection(-1)
                    event.accepted = true
                }
                Keys.onReturnPressed: function(event) {
                    root.choose(currentIndex)
                    event.accepted = true
                }
                Keys.onEnterPressed: function(event) {
                    root.choose(currentIndex)
                    event.accepted = true
                }
                Keys.onEscapePressed: function(event) {
                    popup.close()
                    event.accepted = true
                }
            }
        }
    }
}
