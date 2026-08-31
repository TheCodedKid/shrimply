import QtQuick

Item {
    id: root
    required property var target
    required property var typoBackend
    required property int typoIndex
    readonly property int start: typoBackend.typoStart(typoIndex)
    readonly property int end: start + typoBackend.typoLength(typoIndex)
    readonly property int safeStart: Math.max(0, Math.min(start, target.length))
    readonly property int safeEnd: Math.max(safeStart, Math.min(end, target.length))
    readonly property rect startRect: target.positionToRectangle(safeStart)
    readonly property rect endRect: target.positionToRectangle(safeEnd)

    x: startRect.x
    y: startRect.y + startRect.height - 2
    width: Math.max(2, endRect.x - startRect.x)
    height: 2
    visible: start >= 0 && end <= target.length && Math.abs(startRect.y - endRect.y) < 1

    Rectangle {
        anchors.fill: parent
        color: "#e01b24"
    }
}
