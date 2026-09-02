import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

ControlRow {
    id: root
    property bool active: false
    property bool busy: false
    property string tooltip: ""
    signal toggled(bool active)

    RowLayout {
        spacing: 6

        BusyIndicator {
            visible: root.busy
            running: visible
            implicitWidth: 18
            implicitHeight: 18
        }

        Switch {
            id: toggle
            Layout.alignment: Qt.AlignRight
            checked: root.active
            ToolTip.visible: hovered && root.tooltip.length > 0
            ToolTip.text: ComponentTranslations.text(root.tooltip)
            onClicked: root.toggled(checked)
        }
    }
}
