import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

RowLayout {
    id: root
    property real first: 0
    property real second: 0
    property real minimum: -1000000
    property real maximum: 1000000
    property real dragStep: 1
    property int digits: 2
    property string firstPrefix: ""
    property string secondPrefix: ""
    property string unitName: ""
    property bool enableLock: false
    signal firstEdited(real value)
    signal secondEdited(real value)
    signal firstCommitted(real value)
    signal secondCommitted(real value)
    spacing: 6

    NumberGroupBackend {
        id: group
        onFirstChanged: {
            firstPicker.value = first
            root.first = first
        }
        onSecondChanged: {
            secondPicker.value = second
            root.second = second
        }
        onValueEdited: function(axis, value) {
            if (axis === 0)
                root.firstEdited(value)
            else if (axis === 1)
                root.secondEdited(value)
        }
    }
    Component.onCompleted: {
        group.configure(first, second, 0, 2, enableLock)
        group.setBounds(minimum, maximum)
    }
    onFirstChanged: group.setExternalValue(0, first)
    onSecondChanged: group.setExternalValue(1, second)
    onMinimumChanged: group.setBounds(minimum, maximum)
    onMaximumChanged: group.setBounds(minimum, maximum)
    onEnableLockChanged: group.updateLocked(enableLock && lock.checked)

    ToolButton {
        id: lock
        visible: root.enableLock
        checkable: true
        checked: true
        icon.source: checked
            ? "qrc:/qt/qml/dev/shrimply/components/icons/padlock.svg"
            : "qrc:/qt/qml/dev/shrimply/components/icons/padlock-open.svg"
        icon.color: palette.buttonText
        display: AbstractButton.IconOnly
        ToolTip.visible: hovered
        ToolTip.text: ComponentTranslations.text("Lock ratio")
        onToggled: group.updateLocked(checked)
    }
    NumberPicker {
        id: firstPicker
        Layout.fillWidth: true
        value: root.first
        minimum: root.minimum
        maximum: root.maximum
        dragStep: root.dragStep
        digits: root.digits
        prefix: root.firstPrefix
        unitName: root.unitName
        onEdited: function(value) { group.edit(0, value) }
        onCommitted: function(value) { root.firstCommitted(value) }
    }
    NumberPicker {
        id: secondPicker
        Layout.fillWidth: true
        value: root.second
        minimum: root.minimum
        maximum: root.maximum
        dragStep: root.dragStep
        digits: root.digits
        prefix: root.secondPrefix
        unitName: root.unitName
        onEdited: function(value) { group.edit(1, value) }
        onCommitted: function(value) { root.secondCommitted(value) }
    }
}
