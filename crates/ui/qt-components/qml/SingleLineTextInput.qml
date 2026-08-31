import QtQuick
import dev.shrimply.components
import QtQuick.Controls

TextField {
    id: root
    property string value: ""
    property int maximumLength: 0
    property int activeTypo: -1
    signal edited(string value)
    signal committed(string value)

    TextInputBackend {
        id: backend
        onChanged: function(value) { root.value = value; root.edited(value) }
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
            editor: root
            typoBackend: backend
            typoIndex: index
        }
    }
    ToolTip.visible: root.activeTypo >= 0 && typoHover.hovered
    ToolTip.text: backend.typoMessage(root.activeTypo)
    HoverHandler {
        id: typoHover
        onPointChanged: root.activeTypo = backend.typoAt(root.positionAt(point.position.x))
    }
    TapHandler {
        acceptedButtons: Qt.RightButton
        onTapped: function(eventPoint) {
            root.activeTypo = backend.typoAt(root.positionAt(eventPoint.position.x))
            if (root.activeTypo >= 0)
                typoMenu.popup()
        }
    }
    Menu {
        id: typoMenu
        Instantiator {
            model: 6
            delegate: MenuItem {
                required property int index
                text: backend.typoCorrection(root.activeTypo, index)
                visible: text.length > 0
                onTriggered: root.text = backend.applyCorrection(root.activeTypo, index)
            }
            onObjectAdded: function(index, object) { typoMenu.insertItem(index, object) }
            onObjectRemoved: function(index, object) { typoMenu.removeItem(object) }
        }
    }
}
