pragma ComponentBehavior: Bound

import QtQuick
import dev.shrimply.components
import QtQuick.Layouts

InspectorProperty {
    id: root
    property real initialGraphValue: 0
    property real graphValue: initialGraphValue
    property real initialSecondValue: 0
    property bool paired: false
    property string externalClipboardMarker: ""
    property real firstValue: initialGraphValue
    property real secondValue: initialSecondValue
    property string expressionValue: ""
    property string expressionOutput: ""
    property string expressionError: ""
    readonly property int graphComponent: editRouter.activeComponent
    signal graphLoaded()
    signal graphEditFinished()
    signal graphPlaybackToggled()
    signal graphPlayheadChanged(int component, var numerator, var denominator)
    signal graphKeysMoved(int component, var oldTimes, var times, var values)
    signal graphKeysDeleted(int component, var times)
    signal graphKeyAdded(int component, var numerator, var denominator)
    signal graphCopyRequested(int component, var times)
    signal graphPasteRequested(int component, var numerator, var denominator)
    signal graphInterpolationChanged(int component, string ownerId, int interpolation)
    signal expressionEdited(string value)
    signal expressionCommitted(string value)
    signal baseValueEdited(real value)
    signal basePairEdited(real first, real second)
    signal keyframeValueEdited(real value)
    signal keyframePairEdited(real first, real second, int component)
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
        if (graphLoader.graph)
            graphLoader.graph.configureValue(value)
        root.graphValue = value
        root.graphReset(0, value)
    }

    function resetPair(first, second) {
        editRouter.selectComponent(0)
        editRouter.configurePair(first, second)
        root.firstValue = first
        root.secondValue = second
        if (graphLoader.graph)
            graphLoader.graph.configurePair(first, second, 0)
        root.graphReset(0, first)
        root.graphReset(1, second)
    }

    function replaceRawGraphComponent(component, pointTimes, pointValues, segments, staticValue) {
        if (graphLoader.graph)
            graphLoader.graph.replaceRawGraph(
                component, pointTimes, pointValues, segments, staticValue)
    }
    function replaceStepGraphComponent(component, times, values) {
        if (graphLoader.graph)
            graphLoader.graph.replaceStepGraph(component, times, values)
    }
    function reconcileStepGraphMoves(component, oldTimes, rawTimes, times) {
        if (graphLoader.graph)
            graphLoader.graph.reconcileStepMoves(component, oldTimes, rawTimes, times)
    }
    function rollbackStepGraphMoves(component, oldTimes, rawTimes) {
        if (graphLoader.graph)
            graphLoader.graph.rollbackStepMoves(component, oldTimes, rawTimes)
    }
    function replaceSpeedGraphComponent(component, keys, segments, staticValue) {
        if (graphLoader.graph)
            graphLoader.graph.replaceSpeedGraph(component, keys, segments, staticValue)
    }
    function setGraphRange(startNumerator, startDenominator, endNumerator, endDenominator) {
        if (graphLoader.graph)
            graphLoader.graph.setRange(
                startNumerator, startDenominator, endNumerator, endDenominator)
    }
    function setGraphFrameStep(numerator, denominator) {
        if (graphLoader.graph)
            graphLoader.graph.setFrameStep(numerator, denominator)
    }
    function setGraphPlayhead(numerator, denominator) {
        if (graphLoader.graph)
            graphLoader.graph.setPlayhead(numerator, denominator)
    }
    function setGraphSnapping(enabled, radiusPx) {
        if (graphLoader.graph)
            graphLoader.graph.setSnapping(enabled, radiusPx)
    }
    function setGraphExternalClipboard(enabled) {
        if (graphLoader.graph)
            graphLoader.graph.setExternalClipboard(enabled)
    }
    function copyExternalClipboardMarker() {
        if (root.externalClipboardMarker.length === 0)
            return
        clipboardBridge.text = root.externalClipboardMarker
        clipboardBridge.selectAll()
        clipboardBridge.copy()
        clipboardBridge.deselect()
    }

    Component.onCompleted: {
        editRouter.setModes(root.keyframes, root.expression)
        if (root.paired)
            root.resetPair(initialGraphValue, initialSecondValue)
        else if (!root.keyframes)
            root.resetValue(initialGraphValue)
        else
            root.graphValue = initialGraphValue
    }
    onKeyframesChanged: editRouter.setModes(keyframes, expression)
    onExpressionChanged: editRouter.setModes(keyframes, expression)
    onInitialGraphValueChanged: {
        if (!keyframes && graphLoader.graph)
            graphLoader.graph.configureValue(initialGraphValue)
        graphValue = initialGraphValue
    }
    onInitialSecondValueChanged: if (paired) {
        editRouter.configurePair(initialGraphValue, initialSecondValue)
        firstValue = initialGraphValue
        secondValue = initialSecondValue
        if (graphLoader.graph)
            graphLoader.graph.configurePair(
                initialGraphValue, initialSecondValue, graphComponent)
    }
    onGraphComponentChanged: if (graphLoader.graph)
        graphLoader.graph.activateComponent(graphComponent)

    LayeredPropertyBackend {
        id: editRouter
        onBaseEdited: function(value) { root.baseValueEdited(value) }
        onBasePairEdited: function(first, second) {
            root.firstValue = first
            root.secondValue = second
            root.basePairEdited(first, second)
        }
        onKeyframeEdited: function(value) {
            if (graphLoader.graph)
                graphLoader.graph.configureCurrentValue(value)
            root.keyframeValueEdited(value)
        }
        onKeyframePairEdited: function(first, second, component, firstChanged, secondChanged) {
            root.firstValue = first
            root.secondValue = second
            if (graphLoader.graph)
                graphLoader.graph.editPair(
                    first, second, component, firstChanged, secondChanged)
            root.keyframePairEdited(first, second, component)
        }
    }

    keyframeContent: Loader {
        id: graphLoader
        readonly property FrameGraph graph: item as FrameGraph
        active: root.keyframes
        Layout.fillWidth: true
        onLoaded: root.graphLoaded()
        sourceComponent: Component {
            FrameGraph {
                onGraphValueChanged: {
                    root.graphValue = graphValue
                    if (root.graphComponent === 0)
                        root.firstValue = graphValue
                    else
                        root.secondValue = graphValue
                }
                onTogglePlayback: root.graphPlaybackToggled()
                onEditFinished: root.graphEditFinished()
                onPlayheadChanged: function(component, numerator, denominator) {
                    root.graphPlayheadChanged(component, numerator, denominator)
                }
                onKeysMoved: function(component, oldTimes, times, values) {
                    root.graphKeysMoved(component, oldTimes, times, values)
                }
                onKeysDeleted: function(component, times) {
                    root.graphKeysDeleted(component, times)
                }
                onKeyAdded: function(component, numerator, denominator, value) {
                    root.graphKeyAdded(component, numerator, denominator)
                }
                onCopyRequested: function(component, times) {
                    root.graphCopyRequested(component, times)
                }
                onPasteRequested: function(component, numerator, denominator) {
                    clipboardBridge.text = ""
                    clipboardBridge.paste()
                    if (root.externalClipboardMarker.length === 0
                            || clipboardBridge.text === root.externalClipboardMarker) {
                        root.graphPasteRequested(component, numerator, denominator)
                    }
                }
                onInterpolationChanged: function(component, ownerId, interpolation) {
                    root.graphInterpolationChanged(component, ownerId, interpolation)
                }
            }
        }
    }
    TextEdit {
        id: clipboardBridge
        visible: false
    }
    expressionContent: Loader {
        active: root.expression
        Layout.fillWidth: true
        sourceComponent: Component {
            ExpressionEditor {
                value: root.expressionValue
                output: root.expressionOutput
                error: root.expressionError
                onEdited: function(value) { root.expressionEdited(value) }
                onCommitted: function(value) { root.expressionCommitted(value) }
            }
        }
    }
}
