import QtQuick

Item {
    id: root
    required property var editor
    required property var typoBackend
    required property int typoIndex
    readonly property int start: typoBackend.typoStart(typoIndex)
    readonly property int end: start + typoBackend.typoLength(typoIndex)
    readonly property rect startRect: editor.positionToRectangle(start)
    readonly property rect endRect: editor.positionToRectangle(end)

    x: startRect.x
    y: startRect.y + startRect.height - 2
    width: Math.max(2, endRect.x - startRect.x)
    height: 2
    visible: start >= 0 && Math.abs(startRect.y - endRect.y) < 1

    Rectangle {
        anchors.fill: parent
        color: "#e01b24"
    }
}
