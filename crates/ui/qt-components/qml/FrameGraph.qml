import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.shrimply.components

FocusScope {
    id: root
    signal togglePlayback()
    readonly property real graphValue: graph.graphValue

    function editValue(value) { graph.editGraphValue(value) }
    function configureValue(value) { graph.configureGraphValue(value) }

    implicitWidth: 640
    implicitHeight: controls.implicitHeight + graph.implicitHeight

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        RowLayout {
            id: controls
            Layout.fillWidth: true
            spacing: 6

            Item {
                Layout.fillWidth: true
            }
            ToolButton {
                enabled: graph.canPrevious
                icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/previous.svg"
                icon.color: palette.buttonText
                display: AbstractButton.IconOnly
                ToolTip.visible: hovered
                ToolTip.text: "Previous keyframe"
                onClicked: {
                    graph.previousKey()
                    graph.forceActiveFocus()
                }
            }
            ToolButton {
                icon.source: graph.keyAtPlayhead
                             ? "qrc:/qt/qml/dev/shrimply/components/icons/remove.svg"
                             : "qrc:/qt/qml/dev/shrimply/components/icons/add.svg"
                icon.color: palette.buttonText
                display: AbstractButton.IconOnly
                ToolTip.visible: hovered
                ToolTip.text: graph.keyAtPlayhead
                              ? "Delete keyframe at playhead"
                              : "Add keyframe at playhead"
                onClicked: {
                    graph.toggleKey()
                    graph.forceActiveFocus()
                }
            }
            ToolButton {
                enabled: graph.canNext
                icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/next.svg"
                icon.color: palette.buttonText
                display: AbstractButton.IconOnly
                ToolTip.visible: hovered
                ToolTip.text: "Next keyframe"
                onClicked: {
                    graph.nextKey()
                    graph.forceActiveFocus()
                }
            }
        }

        FrameGraphItem {
            id: graph
            Layout.fillWidth: true
            Layout.fillHeight: true
            implicitHeight: 132
            mirrorVertically: true
            activeFocusOnTab: true
            onTogglePlayback: root.togglePlayback()

            Keys.onPressed: function(event) {
                let key = -1
                if ((event.modifiers & Qt.ControlModifier) && event.key === Qt.Key_C)
                    key = 7
                else if ((event.modifiers & Qt.ControlModifier) && event.key === Qt.Key_V)
                    key = 8
                else if (event.key === Qt.Key_Left) key = 0
                else if (event.key === Qt.Key_Right) key = 1
                else if (event.key === Qt.Key_Home) key = 2
                else if (event.key === Qt.Key_End) key = 3
                else if (event.key === Qt.Key_Plus || event.key === Qt.Key_Equal) key = 4
                else if (event.key === Qt.Key_Minus) key = 5
                else if (event.key === Qt.Key_Delete || event.key === Qt.Key_Backspace) key = 6
                else if (event.key === Qt.Key_Space) key = 9
                if (key >= 0) {
                    graph.handleKey(key)
                    event.accepted = true
                }
            }

            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
                hoverEnabled: true
                preventStealing: true

                onPressed: function(mouse) {
                    graph.forceActiveFocus()
                    const button = mouse.button === Qt.LeftButton ? 0
                                 : mouse.button === Qt.MiddleButton ? 1 : 2
                    if (button !== 2)
                        graph.begin(
                            button, mouse.x, mouse.y,
                            (mouse.modifiers & Qt.ControlModifier) !== 0,
                            (mouse.modifiers & Qt.ShiftModifier) !== 0)
                }
                onPositionChanged: function(mouse) {
                    graph.pointerMoved(mouse.x, mouse.y)
                    if (pressedButtons !== Qt.NoButton)
                        graph.updatePointer(mouse.x, mouse.y)
                }
                onReleased: function(mouse) {
                    if (mouse.button === Qt.RightButton) {
                        const selected = graph.begin(
                            2, mouse.x, mouse.y,
                            (mouse.modifiers & Qt.ControlModifier) !== 0,
                            (mouse.modifiers & Qt.ShiftModifier) !== 0)
                        if (selected >= 0) {
                            interpolationAnchor.x = mouse.x
                            interpolationAnchor.y = controls.height + mouse.y
                            interpolationMenu.selected = selected
                            interpolationMenu.popup(interpolationAnchor, 0, 0)
                        }
                    }
                    graph.endPointer()
                }
                onCanceled: {
                    graph.endPointer()
                }
                onExited: if (pressedButtons === Qt.NoButton) graph.pointerLeft()
                onWheel: function(wheel) {
                    wheel.accepted = graph.scroll(
                        wheel.pixelDelta, wheel.angleDelta,
                        wheel.x, wheel.y,
                        (wheel.modifiers & Qt.ControlModifier) !== 0)
                }
            }
        }
    }

    Item {
        id: interpolationAnchor
        width: 1
        height: 1
    }

    Menu {
        id: interpolationMenu
        property int selected: -1
        popupType: Popup.Window

        Repeater {
            model: graph.interpolationCount
            MenuItem {
                required property int index
                text: graph.interpolationLabel(index)
                checkable: true
                checked: interpolationMenu.selected === index
                onTriggered: graph.setInterpolation(index)
            }
        }
    }
}
