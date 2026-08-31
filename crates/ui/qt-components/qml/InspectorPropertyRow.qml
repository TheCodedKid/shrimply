import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ControlRow {
    id: root
    property bool keyframes: false
    property bool expression: false
    property bool keyframeAvailable: true
    property bool expressionAvailable: true
    default property alias editor: holder.data
    signal keyframesToggled(bool enabled)
    signal expressionToggled(bool enabled)

    RowLayout {
        Item {
            id: holder
            Layout.fillWidth: true
            implicitHeight: childrenRect.height
            onChildrenChanged: {
                for (let index = 0; index < children.length; ++index)
                    children[index].width = Qt.binding(function() { return holder.width })
            }
        }
        ToolButton {
            visible: root.keyframeAvailable
            checkable: true
            checked: root.keyframes
            flat: true
            icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/keyframe.svg"
            icon.color: palette.buttonText
            display: AbstractButton.IconOnly
            ToolTip.visible: hovered
            ToolTip.text: qsTr("Keyframes")
            onToggled: {
                root.keyframes = checked
                root.keyframesToggled(checked)
            }
        }
        ToolButton {
            visible: root.expressionAvailable
            checkable: true
            checked: root.expression
            flat: true
            icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/code.svg"
            icon.color: palette.buttonText
            display: AbstractButton.IconOnly
            ToolTip.visible: hovered
            ToolTip.text: qsTr("Expression")
            onToggled: {
                root.expression = checked
                root.expressionToggled(checked)
            }
        }
    }
}
