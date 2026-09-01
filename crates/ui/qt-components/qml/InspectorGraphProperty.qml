import QtQuick
import dev.shrimply.components
import QtQuick.Layouts

InspectorProperty {
    id: root
    property real initialGraphValue: 0
    property real graphValue: initialGraphValue
    property real initialSecondValue: 0
    property bool paired: false
    property real firstValue: initialGraphValue
    property real secondValue: initialSecondValue
    readonly property int graphComponent: editRouter.activeComponent
    property alias expressionValue: expressionEditor.value
    property alias expressionOutput: expressionEditor.output
    signal graphPlaybackToggled()
    signal expressionEdited(string value)
    signal baseValueEdited(real value)
    signal basePairEdited(real first, real second)
    signal graphReset(int component, real value)

    function editValue(value) {
        editRouter.setModes(root.keyframes, root.expression)
        editRouter.editValue(value)
    }

    function editPair(first, second, component) {
        editRouter.setModes(root.keyframes, root.expression)
        editRouter.editPair(first, second, component)
    }

    function resetValue(value) {
        graph.configureValue(value)
        root.graphValue = value
        root.graphReset(0, value)
    }

    function resetPair(first, second) {
        editRouter.selectComponent(0)
        editRouter.configurePair(first, second)
        root.firstValue = first
        root.secondValue = second
        graph.configurePair(first, second, 0)
        root.graphReset(0, first)
        root.graphReset(1, second)
    }

    function replaceRawGraphComponent(component, pointTimes, pointValues, segments, staticValue) {
        graph.replaceRawGraph(component, pointTimes, pointValues, segments, staticValue)
    }
    function setGraphRange(startNumerator, startDenominator, endNumerator, endDenominator) {
        graph.setRange(startNumerator, startDenominator, endNumerator, endDenominator)
    }
    function setGraphFrameStep(numerator, denominator) {
        graph.setFrameStep(numerator, denominator)
    }
    function setGraphPlayhead(numerator, denominator) {
        graph.setPlayhead(numerator, denominator)
    }
    function setGraphSnapping(enabled, radiusPx) { graph.setSnapping(enabled, radiusPx) }

    Component.onCompleted: {
        editRouter.setModes(root.keyframes, root.expression)
        if (root.paired)
            root.resetPair(initialGraphValue, initialSecondValue)
        else
            root.resetValue(initialGraphValue)
    }
    onKeyframesChanged: editRouter.setModes(keyframes, expression)
    onExpressionChanged: editRouter.setModes(keyframes, expression)
    onGraphComponentChanged: graph.activateComponent(graphComponent)

    LayeredPropertyBackend {
        id: editRouter
        onBaseEdited: function(value) { root.baseValueEdited(value) }
        onBasePairEdited: function(first, second) {
            root.firstValue = first
            root.secondValue = second
            root.basePairEdited(first, second)
        }
        onKeyframeEdited: function(value) { graph.editValue(value) }
        onKeyframePairEdited: function(first, second, component, firstChanged, secondChanged) {
            root.firstValue = first
            root.secondValue = second
            graph.editPair(first, second, component, firstChanged, secondChanged)
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
