import QtQuick
import QtQuick.Controls

Control {
    id: root
    property alias text: label.text
    implicitHeight: label.implicitHeight + topPadding + bottomPadding
    leftPadding: 8
    rightPadding: 8
    topPadding: 7
    bottomPadding: 7

    contentItem: TextEdit {
        id: label
        readOnly: true
        activeFocusOnPress: false
        textFormat: Text.PlainText
        selectByMouse: true
        wrapMode: TextEdit.NoWrap
        color: root.palette.text
    }

    background: Rectangle {
        color: "transparent"
    }
}
