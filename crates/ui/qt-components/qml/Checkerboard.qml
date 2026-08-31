import QtQuick

Item {
    id: root
    property int cellSize: 8
    readonly property int columns: Math.ceil(width / cellSize)
    readonly property int rows: Math.ceil(height / cellSize)

    Rectangle {
        anchors.fill: parent
        color: "#b8b8b8"
    }
    Repeater {
        model: root.columns * root.rows
        Rectangle {
            required property int index
            x: (index % root.columns) * root.cellSize
            y: Math.floor(index / root.columns) * root.cellSize
            width: root.cellSize
            height: root.cellSize
            color: ((index % root.columns) + Math.floor(index / root.columns)) % 2
                   ? "#b8b8b8" : "#e6e6e6"
        }
    }
}
