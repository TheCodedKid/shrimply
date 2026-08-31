import QtQuick

Item {
    id: root
    property real radius: 10

    implicitWidth: radius * 2
    implicitHeight: implicitWidth

    Rectangle {
        x: 0
        y: 1
        width: parent.width
        height: parent.height
        radius: width / 2
        color: Qt.rgba(0, 0, 0, 0.35)
        antialiasing: true
    }
    Rectangle {
        anchors.fill: parent
        radius: width / 2
        color: "#ebebeb"
        border.color: Qt.rgba(0, 0, 0, 0.35)
        antialiasing: true
    }
}
