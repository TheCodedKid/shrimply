import QtQuick

Item {
    id: root
    property int cellSize: 8
    property real originX: 0
    property real originY: 0
    readonly property real phaseX: ((originX % cellSize) + cellSize) % cellSize
    readonly property real phaseY: ((originY % cellSize) + cellSize) % cellSize
    readonly property int firstColumn: Math.floor(originX / cellSize)
    readonly property int firstRow: Math.floor(originY / cellSize)
    readonly property int columns: Math.max(0, Math.ceil((width + phaseX) / cellSize))
    readonly property int rows: Math.max(0, Math.ceil((height + phaseY) / cellSize))

    Rectangle {
        anchors.fill: parent
        color: "#b8b8b8"
    }
    Repeater {
        model: root.columns * root.rows
        Rectangle {
            required property int index
            readonly property int column: index % root.columns
            readonly property int row: Math.floor(index / root.columns)
            x: column * root.cellSize - root.phaseX
            y: row * root.cellSize - root.phaseY
            width: root.cellSize
            height: root.cellSize
            color: (column + root.firstColumn + row + root.firstRow) % 2
                   ? "#b8b8b8" : "#e6e6e6"
        }
    }
}
