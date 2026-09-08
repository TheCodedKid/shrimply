import QtQuick
import QtQuick.Effects

Item {
    id: root
    property color color: "transparent"
    property real radius: 4
    property color borderColor: "transparent"
    property bool checkerboard: color.a < 1
    property int checkerCellSize: 8
    property real checkerOriginX: 0
    property real checkerOriginY: 0
    property alias fillGradient: fill.gradient

    Item {
        id: contents
        anchors.fill: parent
        visible: false
        layer.enabled: true
        layer.samples: 4

        Checkerboard {
            anchors.fill: parent
            visible: root.checkerboard
            cellSize: root.checkerCellSize
            originX: root.checkerOriginX
            originY: root.checkerOriginY
        }
        Rectangle {
            id: fill
            anchors.fill: parent
            color: root.color
        }
        Rectangle {
            anchors.fill: parent
            color: "transparent"
            border.color: root.borderColor
        }
    }

    Rectangle {
        id: mask
        anchors.fill: parent
        visible: false
        layer.enabled: true
        layer.samples: 4
        radius: root.radius
        color: "white"
        antialiasing: true
    }

    MultiEffect {
        anchors.fill: parent
        source: contents
        maskEnabled: true
        maskSource: mask
    }
}
