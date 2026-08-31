import QtQuick
import dev.shrimply.components
import QtQuick.Layouts

InspectorProperty {
    id: root
    property real initialGraphValue: 0
    property real graphValue: initialGraphValue
    property real initialSecondValue: 0
    property real firstValue: initialGraphValue
    property real secondValue: initialSecondValue
    property int graphComponent: 0
    property alias expressionValue: expressionEditor.value
    property alias expressionOutput: expressionEditor.output
    signal graphPlaybackToggled()
    signal expressionEdited(string value)
    signal baseValueEdited(real value)
    signal basePairEdited(real first, real second)

    function editValue(value) {
        editRouter.setModes(root.keyframes, root.expression)
        editRouter.editValue(value)
    }

    function editPair(first, second, component) {
        root.graphComponent = component
        editRouter.setModes(root.keyframes, root.expression)
        editRouter.editPair(first, second, component)
    }

    function resetValue(value) {
        graph.configureValue(value)
        root.graphValue = value
    }

    function resetPair(first, second) {
        root.firstValue = first
        root.secondValue = second
        graph.configureValue(root.graphComponent === 0 ? first : second)
    }

    Component.onCompleted: {
        editRouter.setModes(root.keyframes, root.expression)
        graph.configureValue(initialGraphValue)
        root.graphValue = initialGraphValue
        root.firstValue = initialGraphValue
        root.secondValue = initialSecondValue
    }
    onKeyframesChanged: editRouter.setModes(keyframes, expression)
    onExpressionChanged: editRouter.setModes(keyframes, expression)

    LayeredPropertyBackend {
        id: editRouter
        onBaseEdited: function(value) { root.baseValueEdited(value) }
        onBasePairEdited: function(first, second) {
            root.firstValue = first
            root.secondValue = second
            root.basePairEdited(first, second)
        }
        onKeyframeEdited: function(value) { graph.editValue(value) }
        onKeyframePairEdited: function(first, second, graphValue) {
            root.firstValue = first
            root.secondValue = second
            graph.editValue(graphValue)
        }
    }

    keyframeContent: FrameGraph {
        id: graph
        Layout.fillWidth: true
        onGraphValueChanged: {
            root.graphValue = graphValue
            if (root.graphComponent === 0)
                root.firstValue = graphValue
            else
                root.secondValue = graphValue
        }
        onTogglePlayback: root.graphPlaybackToggled()
    }
    expressionContent: ExpressionEditor {
        id: expressionEditor
        Layout.fillWidth: true
        onEdited: function(value) { root.expressionEdited(value) }
    }
}
