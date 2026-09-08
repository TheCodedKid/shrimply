import QtQuick
import dev.shrimply.components

InspectorGraphProperty {
    id: root
    tripled: true
    property alias minimum: editor.minimum
    property alias maximum: editor.maximum
    property alias dragStep: editor.dragStep
    property alias digits: editor.digits
    property alias widthCharacters: editor.widthCharacters
    property alias prefixes: editor.prefixes
    property alias unitName: editor.unitName
    property alias enableLock: editor.enableLock
    signal tripleCommitted()
    Number3Picker {
        id: editor
        first: root.firstValue
        second: root.secondValue
        third: root.thirdValue
        onValuesEdited: function(first, second, third, component) {
            root.editTriple(first, second, third, component)
        }
        onCommitted: root.tripleCommitted()
    }
}
