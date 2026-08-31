import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

ControlRow {
    id: root
    property bool active: false
    property string tooltip: ""
    signal toggled(bool active)

    Item {
        implicitHeight: toggle.implicitHeight

        Switch {
            id: toggle
            anchors.right: parent.right
            checked: root.active
            ToolTip.visible: hovered && root.tooltip.length > 0
            ToolTip.text: ComponentTranslations.text(root.tooltip)
            onToggled: {
                root.active = checked
                root.toggled(checked)
            }
        }
    }
}
