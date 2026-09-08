import QtQml
import dev.shrimply.components.demo

QtObject {
    required property var target

    function load(component, value) {
        target.replaceRawGraphComponent(
            component,
            DemoLogic.graphPointTimes(),
            DemoLogic.graphPointValues(value),
            DemoLogic.graphSegments(value),
            value)
        target.setGraphRange(0, 1, 4, 1)
        target.setGraphFrameStep(1, 30)
        target.setGraphPlayhead(3, 2)
        target.setGraphSnapping(true, 8)
    }
}
