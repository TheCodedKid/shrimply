import QtQuick
import dev.shrimply.components
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
    clip: true

    contentItem: RowLayout {
        id: content
        spacing: 6

        TextEdit {
            id: label
            Layout.fillWidth: true
            Layout.minimumWidth: 0
            readOnly: true
            activeFocusOnPress: true
            activeFocusOnTab: false
            textFormat: Text.PlainText
            selectByMouse: true
            selectByKeyboard: true
            persistentSelection: true
            wrapMode: TextEdit.NoWrap
            color: root.palette.text
            Accessible.role: Accessible.StaticText
            Accessible.name: text
            Accessible.editable: false
            Accessible.selectableText: true

            TapHandler {
                acceptedButtons: Qt.RightButton
                onTapped: function(eventPoint) {
                    contextMenu.openAt(eventPoint.position.x, eventPoint.position.y, -1)
                }
            }
            TextContextMenu {
                id: contextMenu
                editor: label
            }
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
