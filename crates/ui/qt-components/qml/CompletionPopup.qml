import QtQuick
import QtQuick.Controls

Popup {
    id: root
    property var candidates: []
    property int currentIndex: 0
    signal accepted(string candidate)

    parent: Overlay.overlay
    width: 220
    height: Math.min(180, choices.contentHeight) + topPadding + bottomPadding
    padding: 2
    focus: false
    modal: false
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose

    function showAt(target, rectangle) {
        const below = target.mapToItem(parent, rectangle.x,
            rectangle.y + rectangle.height)
        const above = target.mapToItem(parent, rectangle.x, rectangle.y)
        x = Math.max(0, Math.min(below.x, parent.width - width))
        y = below.y + height <= parent.height
            ? below.y : Math.max(0, above.y - height)
        currentIndex = 0
        open()
    }

    function moveSelection(offset) {
        if (candidates.length === 0)
            return
        currentIndex = (currentIndex + offset + candidates.length)
            % candidates.length
        choices.positionViewAtIndex(currentIndex, ListView.Contain)
    }

    function acceptCurrent() {
        if (currentIndex >= 0 && currentIndex < candidates.length)
            accepted(candidates[currentIndex])
    }

    background: Rectangle {
        color: root.palette.window
        border.color: root.palette.mid
        radius: 3
    }

    contentItem: ListView {
        id: choices
        clip: true
        model: root.candidates
        currentIndex: root.currentIndex
        delegate: ItemDelegate {
            required property int index
            required property string modelData
            width: choices.width
            height: implicitHeight
            text: modelData
            highlighted: index === root.currentIndex
            focusPolicy: Qt.NoFocus
            onHoveredChanged: if (hovered) root.currentIndex = index
            onClicked: root.accepted(modelData)
        }
    }
}
