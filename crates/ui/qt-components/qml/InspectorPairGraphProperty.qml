import QtQuick
import dev.shrimply.components

InspectorGraphProperty {
    id: root
    paired: true
    property alias minimum: editor.minimum
    property alias maximum: editor.maximum
    property alias dragStep: editor.dragStep
    property alias digits: editor.digits
    property alias firstPrefix: editor.firstPrefix
    property alias secondPrefix: editor.secondPrefix
    property alias unitName: editor.unitName
    property alias enableLock: editor.enableLock
    Number2Picker {
        id: editor
        first: root.firstValue
        second: root.secondValue
        onValuesEdited: function(first, second, component) {
            root.editPair(first, second, component)
        }
    }
}
