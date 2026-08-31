import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Frame {
    id: root
    property string title: ""
    property bool expanded: true
    default property alias controls: body.content
    signal resetRequested()
    padding: 0
    implicitHeight: content.implicitHeight

    contentItem: ColumnLayout {
        id: content
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.rightMargin: 8
            Layout.topMargin: 6
            Layout.bottomMargin: 6
            spacing: 4

            ToolButton {
                icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/disclosure.svg"
                icon.color: palette.buttonText
                display: AbstractButton.IconOnly
                rotation: root.expanded ? 90 : 0
                Behavior on rotation { NumberAnimation { duration: 180 } }
                ToolTip.visible: hovered
                ToolTip.text: root.expanded ? qsTr("Collapse") : qsTr("Expand")
                onClicked: root.expanded = !root.expanded
            }
            Label {
                text: root.title
                font.bold: true
                Layout.fillWidth: true
            }
            ToolButton {
                icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/reset.svg"
                icon.color: palette.buttonText
                display: AbstractButton.IconOnly
                ToolTip.visible: hovered
                ToolTip.text: qsTr("Reset")
                onClicked: root.resetRequested()
            }
        }

        CollapsibleSection {
            id: body
            expanded: root.expanded
            Layout.fillWidth: true
            Layout.leftMargin: 12
            Layout.rightMargin: 12
            Layout.topMargin: 4
            Layout.bottomMargin: 12
            spacing: 8
        }
    }
}
