import QtQuick
import QtQuick.Layouts

ColumnLayout {
    id: root
    property alias label: row.label
    property alias keyframes: row.keyframes
    property alias expression: row.expression
    property alias keyframeAvailable: row.keyframeAvailable
    property alias expressionAvailable: row.expressionAvailable
    default property alias editor: row.editor
    property alias keyframeContent: keyframeSection.content
    property alias expressionContent: expressionSection.content
    signal keyframesToggled(bool enabled)
    signal expressionToggled(bool enabled)
    spacing: 0

    InspectorPropertyRow {
        id: row
        Layout.fillWidth: true
        onKeyframesToggled: function(enabled) { root.keyframesToggled(enabled) }
        onExpressionToggled: function(enabled) { root.expressionToggled(enabled) }
    }
    CollapsibleSection {
        id: keyframeSection
        expanded: row.keyframes
        Layout.fillWidth: true
        Layout.topMargin: expanded ? 6 : 0
        spacing: 0
    }
    CollapsibleSection {
        id: expressionSection
        expanded: row.expression
        Layout.fillWidth: true
        Layout.topMargin: expanded ? 6 : 0
        spacing: 0
    }
}
