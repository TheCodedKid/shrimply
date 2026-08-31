import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.shrimply.components

ApplicationWindow {
    id: window
    width: 920
    height: 900
    visible: true
    title: "Shrimply Qt Components"
    property string events: ""

    function log(message) {
        events = message + "\n" + events
    }

    Tabs {
        anchors.fill: parent
        titles: ["General", "Info", "Log"]
        icons: ["preferences-system", "dialog-information", "utilities-terminal"]

        ScrollView {
        contentWidth: availableWidth
        ColumnLayout {
            width: parent.width
            spacing: 10

            Label { text: "Inputs"; font.bold: true }
            ControlRow {
                label: "Number"
                NumberPicker {
                    value: 12.5; minimum: -100; maximum: 100; dragStep: 0.25; digits: 2; unitName: "px"
                    onEdited: function(value) { window.log("number changed " + value) }
                    onCommitted: function(value) { window.log("number committed " + value) }
                }
            }
            ControlRow {
                label: "Pair"
                Number2Picker {
                    first: 1920; second: 1080; minimum: 1; maximum: 16384; digits: 0
                    firstPrefix: "W"; secondPrefix: "H"; unitName: "px"; enableLock: true
                    onFirstEdited: function(value) { window.log("pair first " + value) }
                    onSecondEdited: function(value) { window.log("pair second " + value) }
                }
            }
            ControlRow {
                label: "Vector"
                Number3Picker {
                    first: 1; second: 2; third: 3; prefixes: ["X", "Y", "Z"]; enableLock: true
                    onEdited: function(axis, value) { window.log("vector " + axis + " " + value) }
                }
            }
            ControlRow {
                label: "Single line"
                SingleLineTextInput {
                    value: "Editable text"; placeholderText: "Type here"; maximumLength: 40
                    onCommitted: function(value) { window.log("text committed " + value) }
                }
            }
            ControlRow {
                label: "Multiline"
                MultilineTextInput {
                    value: "Try a typo such as teh."; maximumLength: 240; minimumContentHeight: 110
                    onCommitted: function(value) { window.log("multiline committed") }
                }
            }
            ControlRow {
                label: "Searchable dropdown"
                Dropdown {
                    values: ["one", "two", "three", "four", "five", "six", "seven", "eight"]
                    labels: ["One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight"]
                    value: "two"
                    onSelected: function(value) { window.log("selected " + value) }
                }
            }
            ControlRow {
                label: "Number modes"
                RowLayout {
                    NumberPicker {
                        id: numberModeValue
                        Layout.fillWidth: true
                        value: graphEditor.graphValue
                        digits: 2
                        onEdited: function(value) { graphEditor.editValue(value) }
                    }
                        ToolButton {
                        id: keyframeMode
                        checkable: true
                        checked: true
                        icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/keyframe.svg"
                        icon.color: palette.buttonText
                        display: AbstractButton.IconOnly
                        ToolTip.visible: hovered
                        ToolTip.text: "Keyframes"
                        onToggled: {
                            window.log("keyframes " + checked)
                        }
                    }
                    ToolButton {
                        id: expressionMode
                        checkable: true
                        checked: true
                        icon.source: "qrc:/qt/qml/dev/shrimply/components/icons/code.svg"
                        icon.color: palette.buttonText
                        display: AbstractButton.IconOnly
                        ToolTip.visible: hovered
                        ToolTip.text: "Expression"
                        onToggled: {
                            window.log("expression " + checked)
                        }
                    }
                }
            }
            FrameGraph {
                id: graphEditor
                visible: keyframeMode.checked
                Layout.fillWidth: true
                Layout.preferredHeight: visible ? implicitHeight : 0
                onTogglePlayback: window.log("toggle playback")
            }
            Connections {
                target: graphEditor
                function onGraphValueChanged() {
                    numberModeValue.value = graphEditor.graphValue
                }
            }
            ColumnLayout {
                visible: expressionMode.checked
                Layout.fillWidth: true
                Layout.preferredHeight: visible ? 220 : 0
                spacing: 6
                CodeEditor {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 180
                    value: "value * 2.0"
                    onEdited: function(value) {
                        expressionOutput.text = "Output · expression updated (" + value.length + " chars)"
                        window.log("expression edited (" + value.length + " chars)")
                    }
                }
                ReadOnlyField {
                    id: expressionOutput
                    Layout.fillWidth: true
                    text: "Output · 84.0"
                }
            }
            SwitchRow { label: "Enabled"; tooltip: "Toggle this option"; active: true; onToggled: function(value) { window.log("switch " + value) } }
            ControlRow {
                label: "Color"
                ColorPicker { color: "#3584e4cc"; withAlpha: true; onSelected: function(value) { window.log("color " + value) } }
            }
            ControlRow {
                label: "Split"
                SplitButton { text: "Primary"; secondaryText: "Secondary"; onTriggered: window.log("primary"); onSecondaryTriggered: window.log("secondary") }
            }
            ControlRow {
                label: "Progress"
                RowLayout {
                    ProgressButton { text: "Idle" }
                    ProgressButton { text: "Working"; progressState: ProgressButton.Indeterminate }
                    ProgressButton { text: "Half"; progressState: ProgressButton.Progress; progress: 0.5 }
                }
            }
            ControlRow {
                label: "Playback keys"
                PlaybackShortcuts {
                    implicitHeight: 44
                    onTogglePlaying: window.log("toggle playback")
                    onStepPlaybackSpeedForward: window.log("step playback speed")
                    Rectangle {
                        anchors.fill: parent
                        color: "transparent"
                        border.color: palette.mid
                        Label { anchors.centerIn: parent; text: "Click, then press Space or L" }
                    }
                }
            }

        }
    }

        ScrollView {
            contentWidth: availableWidth
            ColumnLayout {
                width: parent.width
                spacing: 10
                Label { text: "Information"; font.bold: true }
                ControlRow {
                    label: "Selected item"
                    ReadOnlyField { text: "Example clip · 00:00:02:00" }
                }
                ControlRow {
                    label: "Component package"
                    ReadOnlyField { text: "shrimply-qt-components" }
                }
                ControlRow {
                    label: "Frame graph"
                    ReadOnlyField { text: "Shared Rust renderer" }
                }
            }
        }

        ScrollView {
            TextArea {
                width: parent.width
                height: parent.height
                readOnly: true
                text: window.events
                font.family: "monospace"
            }
        }
    }
}
