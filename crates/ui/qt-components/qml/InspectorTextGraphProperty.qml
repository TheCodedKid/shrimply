import QtQuick
import dev.shrimply.components

InspectorGraphProperty {
    id: root
    property string textValue: ""
    property int minimumContentHeight: 86
    signal textEdited(string value)
    signal textCommitted(string value)
    signal textInterpolationRequested(string ownerId, real x, real y)
    signal textInterpolationChanged(string ownerId, int interpolation)

    graphValueDrivesEditor: false
    initialGraphValue: 0
    onGraphTextInterpolationRequested: function(component, ownerId, x, y) {
        if (component === 0)
            root.textInterpolationRequested(ownerId, x, y)
    }
    onGraphTextInterpolationChanged: function(component, ownerId, interpolation) {
        if (component === 0)
            root.textInterpolationChanged(ownerId, interpolation)
    }

    MultilineTextInput {
        value: root.textValue
        minimumContentHeight: root.minimumContentHeight
        onEdited: function(value) { root.textEdited(value) }
        onCommitted: function(value) { root.textCommitted(value) }
    }
}
