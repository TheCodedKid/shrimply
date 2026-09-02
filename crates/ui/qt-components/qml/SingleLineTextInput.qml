import QtQuick
import dev.shrimply.components
import QtQuick.Controls

TextField {
    id: root
    property string value: ""
    property int maximumLength: 0
    property int hoverTypo: -1
    signal edited(string value)
    signal committed(string value)

    TextInputBackend {
        id: backend
        onChanged: function(value) { root.edited(value) }
        onCommitted: function(value) { root.committed(value) }
    }
    Component.onCompleted: {
        backend.configure(value, maximumLength)
        text = backend.text
    }
    onValueChanged: if (!activeFocus && value !== backend.text) {
        backend.configure(value, maximumLength)
        text = backend.text
    }
    onTextEdited: {
        const accepted = backend.edit(text)
        if (accepted !== text)
            text = accepted
    }
    onAccepted: backend.commit()
    onActiveFocusChanged: if (!activeFocus) backend.commit()

    Repeater {
        model: backend.typoCount
        TypoUnderline {
            required property int index
            target: root
            typoBackend: backend
            typoIndex: index
        }
    }
    ToolTip.visible: root.hoverTypo >= 0 && typoHover.hovered
    ToolTip.text: backend.typoMessage(root.hoverTypo)
    HoverHandler {
        id: typoHover
        onPointChanged: root.hoverTypo = backend.typoAt(root.positionAt(point.position.x))
    }
    TapHandler {
        acceptedButtons: Qt.RightButton
        onTapped: function(eventPoint) {
            typoMenu.openAt(eventPoint.position.x, eventPoint.position.y,
                backend.typoAt(root.positionAt(eventPoint.position.x)))
        }
    }
    TextContextMenu {
        id: typoMenu
        editor: root
        typoBackend: backend
    }
}
