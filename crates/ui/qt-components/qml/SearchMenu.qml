pragma ComponentBehavior: Bound

import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

Menu {
    id: root
    property var labels: []
    property int selectedIndex: -1
    property bool searchEnabled: true
    property string placeholderText: ComponentTranslations.text("Search")
    property real minimumListHeight: 0
    property real maximumListHeight: 280
    property bool navigationActive: false
    signal activated(int index)

    width: 280
    padding: 6
    focus: true
    popupType: Popup.Window
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent

    function choose(index) {
        if (index < 0 || index >= labels.length)
            return
        activated(index)
        close()
    }

    function moveSelection(direction) {
        navigationActive = true
        const next = backend.nextMatchingIndex(
            root.labels, search.text, choices.currentIndex, direction)
        if (next >= 0) {
            choices.currentIndex = next
            choices.positionViewAtIndex(next, ListView.Contain)
        }
    }

    onOpened: {
        search.clear()
        navigationActive = false
        choices.currentIndex = selectedIndex >= 0
            ? selectedIndex
            : backend.nextMatchingIndex(root.labels, "", -1, 1)
        if (searchEnabled)
            search.forceActiveFocus(Qt.PopupFocusReason)
        else
            choices.forceActiveFocus(Qt.PopupFocusReason)
    }

    SelectorBackend { id: backend }

    contentItem: ColumnLayout {
        spacing: 6

        TextField {
            id: search
            visible: root.searchEnabled
            Layout.fillWidth: true
            placeholderText: root.placeholderText
            selectByMouse: true
            onTextChanged: {
                root.navigationActive = false
                choices.currentIndex = backend.nextMatchingIndex(
                    root.labels, text, -1, 1)
            }
            Keys.onDownPressed: function(event) {
                root.navigationActive = true
                choices.currentIndex = backend.nextMatchingIndex(
                    root.labels, search.text, -1, 1)
                choices.forceActiveFocus(Qt.TabFocusReason)
                event.accepted = true
            }
            Keys.onUpPressed: function(event) {
                root.navigationActive = true
                choices.currentIndex = backend.nextMatchingIndex(
                    root.labels, search.text, -1, -1)
                choices.forceActiveFocus(Qt.TabFocusReason)
                event.accepted = true
            }
            Keys.onEscapePressed: function(event) {
                root.close()
                event.accepted = true
            }
            onAccepted: root.choose(choices.currentIndex)
        }

        ListView {
            id: choices
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(
                root.maximumListHeight,
                Math.max(root.minimumListHeight, contentHeight))
            clip: true
            model: root.labels
            keyNavigationEnabled: false
            delegate: ItemDelegate {
                required property int index
                width: choices.width
                text: root.labels[index]
                visible: backend.matchesQuery(text, search.text)
                height: visible ? implicitHeight : 0
                icon.source: root.selectedIndex === index
                    ? "qrc:/qt/qml/dev/shrimply/components/icons/object-select.svg"
                    : ""
                icon.color: palette.buttonText
                highlighted: root.navigationActive && choices.currentIndex === index
                onHoveredChanged: if (hovered) {
                    root.navigationActive = true
                    choices.currentIndex = index
                }
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
                root.close()
                event.accepted = true
            }
        }
    }
}
