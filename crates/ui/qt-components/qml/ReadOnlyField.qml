import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Control {
    id: root
    property alias text: label.text
    property alias horizontalAlignment: label.horizontalAlignment
    property url actionIconSource
    property string actionText: ""
    signal actionTriggered()
    implicitHeight: content.implicitHeight + topPadding + bottomPadding
    leftPadding: 8
    rightPadding: 8
    topPadding: 7
    bottomPadding: 7

    contentItem: RowLayout {
        id: content
        spacing: 6

        TextEdit {
            id: label
            Layout.fillWidth: true
            readOnly: true
            activeFocusOnPress: false
            textFormat: Text.PlainText
            selectByMouse: true
            wrapMode: TextEdit.NoWrap
            color: root.palette.text
        }

        ToolButton {
            visible: root.actionIconSource.toString().length > 0
            icon.source: root.actionIconSource
            icon.color: palette.buttonText
            text: root.actionText
            display: AbstractButton.IconOnly
            ToolTip.visible: hovered && text.length > 0
            ToolTip.text: text
            Accessible.name: text
            onClicked: root.actionTriggered()
        }
    }

    background: Rectangle {
        color: "transparent"
    }
}
