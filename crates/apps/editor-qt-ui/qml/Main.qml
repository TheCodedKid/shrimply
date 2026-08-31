import QtCore
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import dev.shrimply.editor

ApplicationWindow {
    id: window

    width: 1800
    height: 1100
    minimumWidth: 960
    minimumHeight: 640
    visible: true
    title: backend.projectTitle
    property bool inspectorVisible: true
    property bool timelineVisible: true
    property string destinationTitle
    property string destinationName

    EditorBackend {
        id: backend
    }

    Component.onCompleted: Qt.callLater(backend.begin)

    Timer {
        interval: 16
        repeat: true
        running: true
        onTriggered: backend.poll()
    }

    Connections {
        target: backend

        function onRequestKdenlive() { kdenliveDialog.open() }
        function onRequestOtio() { otioDialog.open() }
        function onRequestRepair() { repairDialog.open() }
        function onRequestDestination(title, suggestedName) {
            window.destinationTitle = title
            window.destinationName = suggestedName
            destinationDialog.open()
        }
        function onRequestWarnings(body) {
            warningDialog.text = body
            warningDialog.open()
        }
        function onRequestLock(pid) {
            lockDialog.pid = pid
            lockDialog.open()
        }
        function onShowError(heading, body) {
            errorDialog.title = heading
            errorDialog.text = body
            errorDialog.open()
        }
        function onShowPlaybackError(body) {
            audioErrorDialog.text = body
            audioErrorDialog.open()
        }
        function onCanceled() { Qt.quit() }
    }

    menuBar: MenuBar {
        visible: backend.ready

        Menu {
            title: qsTr("Project")
            popupType: Popup.Native

            Action { text: qsTr("Save"); shortcut: StandardKey.Save; onTriggered: backend.save() }
            Action { text: qsTr("Save As…"); shortcut: StandardKey.SaveAs }
            MenuSeparator {}
            Action { text: qsTr("Export…"); shortcut: "Ctrl+E" }
            MenuSeparator {}
            Action { text: qsTr("Quit"); shortcut: StandardKey.Quit; onTriggered: Qt.quit() }
        }

        Menu {
            title: qsTr("Edit")
            popupType: Popup.Native

            Action { text: qsTr("Undo"); shortcut: StandardKey.Undo; onTriggered: backend.undo() }
            Action { text: qsTr("Redo"); shortcut: StandardKey.Redo; onTriggered: backend.redo() }
            MenuSeparator {}
            Action { text: qsTr("Preferences…"); shortcut: StandardKey.Preferences }
        }

        Menu {
            id: viewMenu
            title: qsTr("View")
            popupType: Popup.Native
            onAboutToShow: {
                inspectorMenuItem.checked = window.inspectorVisible
                timelineMenuItem.checked = window.timelineVisible
                console.info("View menu synchronized:",
                    "inspectorVisible=" + window.inspectorVisible,
                    "inspectorChecked=" + inspectorMenuItem.checked,
                    "timelineVisible=" + window.timelineVisible,
                    "timelineChecked=" + timelineMenuItem.checked)
            }

            MenuItem {
                id: inspectorMenuItem
                text: qsTr("Inspector")
                checkable: true
                checked: true
                onClicked: {
                    window.inspectorVisible = !window.inspectorVisible
                    checked = window.inspectorVisible
                    Qt.callLater(function() {
                        console.info("Inspector view toggle settled:",
                            "visible=" + window.inspectorVisible,
                            "checked=" + inspectorMenuItem.checked,
                            "paneVisible=" + inspectorPane.visible)
                    })
                }
            }
            MenuItem {
                id: timelineMenuItem
                text: qsTr("Timeline")
                checkable: true
                checked: true
                onClicked: {
                    window.timelineVisible = !window.timelineVisible
                    checked = window.timelineVisible
                    Qt.callLater(function() {
                        console.info("Timeline view toggle settled:",
                            "visible=" + window.timelineVisible,
                            "checked=" + timelineMenuItem.checked,
                            "paneVisible=" + timelinePane.visible)
                    })
                }
            }
            Action { text: qsTr("Fullscreen Preview"); shortcut: "F11" }
        }

        Menu {
            title: qsTr("Help")
            popupType: Popup.Native

            Action { text: qsTr("Keyboard Shortcuts") }
            Action { text: qsTr("About Shrimply") }
        }
    }

    Pane {
        anchors.fill: parent
        visible: !backend.ready

        ColumnLayout {
            anchors.centerIn: parent
            spacing: 14

            BusyIndicator {
                Layout.alignment: Qt.AlignHCenter
                running: true
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                text: qsTr("Loading project…")
                font.pointSize: 18
                font.bold: true
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                text: backend.loadingText
                opacity: 0.7
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        visible: backend.ready
        spacing: 0

        SplitView {
            id: verticalSplit
            Layout.fillWidth: true
            Layout.fillHeight: true
            orientation: Qt.Vertical

            SplitView {
                SplitView.fillWidth: true
                SplitView.fillHeight: true
                SplitView.preferredHeight: 660
                orientation: Qt.Horizontal

                Pane {
                    id: inspectorPane
                    visible: window.inspectorVisible
                    SplitView.preferredWidth: 320
                    SplitView.minimumWidth: 260

                    ColumnLayout {
                        anchors.fill: parent

                        Label { text: qsTr("Inspector"); font.bold: true }
                        Item { Layout.fillHeight: true }
                        Label {
                            Layout.alignment: Qt.AlignCenter
                            text: qsTr("Inspector is not available yet")
                            opacity: 0.65
                        }
                        Item { Layout.fillHeight: true }
                    }
                }

                ColumnLayout {
                    SplitView.fillWidth: true
                    SplitView.fillHeight: true
                    spacing: 0

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        spacing: 0

                        ToolBar {
                            Layout.fillHeight: true
                            Layout.preferredWidth: 44

                            ColumnLayout {
                                anchors.top: parent.top
                                anchors.horizontalCenter: parent.horizontalCenter
                                ToolButton { icon.name: "task-complete"; text: qsTr("Ready"); display: AbstractButton.IconOnly; enabled: false }
                                Label {
                                    Layout.preferredWidth: 44
                                    Layout.preferredHeight: 34
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                    text: backend.frameRateLabel
                                    font.family: backend.fixedFontFamily
                                    ToolTip.visible: fpsHover.hovered
                                    ToolTip.text: qsTr("Frame rate")
                                    HoverHandler { id: fpsHover }
                                }
                                Label {
                                    Layout.preferredWidth: 44
                                    Layout.preferredHeight: 34
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                    text: backend.playbackSpeedLabel
                                    font.family: backend.fixedFontFamily
                                    ToolTip.visible: speedHover.hovered
                                    ToolTip.text: qsTr("Playback speed")
                                    HoverHandler { id: speedHover }
                                }
                                ToolButton { icon.name: "show-guides"; text: qsTr("Guides"); display: AbstractButton.IconOnly }
                                ToolSeparator {}
                                ToolButton { icon.name: "draw-freehand"; text: qsTr("Pen"); display: AbstractButton.IconOnly }
                                ToolButton { icon.name: "fill-color"; text: qsTr("Fill"); display: AbstractButton.IconOnly }
                                ToolButton { icon.name: "transform-move"; text: qsTr("Transform"); display: AbstractButton.IconOnly }
                                ToolButton { icon.name: "draw-eraser"; text: qsTr("Eraser"); display: AbstractButton.IconOnly }
                            }
                        }

                        Loader {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            active: backend.ready
                            sourceComponent: Component {
                                PreviewSurface { anchors.fill: parent }
                            }
                        }
                    }

                    ToolBar {
                        Layout.fillWidth: true

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 8
                            anchors.rightMargin: 8

                            ToolButton { icon.name: "media-seek-backward"; onClicked: backend.stepFrame(-1) }
                            ToolButton {
                                icon.name: backend.playing ? "media-playback-pause" : "media-playback-start"
                                onClicked: backend.togglePlaying()
                            }
                            ToolButton { icon.name: "media-seek-forward"; onClicked: backend.stepFrame(1) }
                            Slider {
                                Layout.fillWidth: true
                                from: 0
                                to: Math.max(1, backend.durationFrame)
                                value: backend.positionFrame
                                onMoved: backend.seekFrame(Math.round(value))
                            }
                            Label {
                                text: backend.timeLabel
                                font.family: backend.fixedFontFamily
                            }
                            ToolButton { icon.name: "view-fullscreen"; text: qsTr("Fullscreen Preview"); display: AbstractButton.IconOnly }
                        }
                    }
                }
            }

            RowLayout {
                id: timelinePane
                visible: window.timelineVisible
                SplitView.fillWidth: true
                SplitView.preferredHeight: 410
                SplitView.minimumHeight: 180
                spacing: 0

                ToolBar {
                    Layout.fillHeight: true
                    Layout.preferredWidth: 44

                    ColumnLayout {
                        anchors.top: parent.top
                        anchors.horizontalCenter: parent.horizontalCenter
                        ToolButton { icon.name: "edit-select"; text: qsTr("Selection"); display: AbstractButton.IconOnly; checked: true }
                        ToolButton { icon.name: "snap"; text: qsTr("Snapping"); display: AbstractButton.IconOnly; checkable: true }
                        ToolButton { icon.name: "edit-cut"; text: qsTr("Cut"); display: AbstractButton.IconOnly; checkable: true }
                    }
                }

                Loader {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    active: backend.ready
                    sourceComponent: Component {
                        TimelineSurface { anchors.fill: parent; focus: true }
                    }
                }

                Loader {
                    Layout.preferredWidth: 54
                    Layout.fillHeight: true
                    active: backend.ready
                    sourceComponent: Component {
                        AudioMeterSurface { anchors.fill: parent }
                    }
                }
            }
        }
    }

    MessageDialog {
        id: kdenliveDialog
        title: qsTr("Convert Kdenlive Project?")
        text: qsTr("Shrimply supports only some Kdenlive features. Unsupported content may be changed or omitted.")
        buttons: MessageDialog.Ok | MessageDialog.Cancel
        onAccepted: backend.confirmKdenlive(true)
        onRejected: backend.confirmKdenlive(false)
    }

    Dialog {
        id: otioDialog
        title: qsTr("OTIO Project Settings")
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel
        onAccepted: backend.chooseOtio(true, otioWidth.value, otioHeight.value, fpsNumerator.value, fpsDenominator.value)
        onRejected: backend.chooseOtio(false, 0, 0, 0, 0)

        GridLayout {
            columns: 2
            Label { text: qsTr("Width") }
            SpinBox { id: otioWidth; from: 1; to: 16384; value: 1920 }
            Label { text: qsTr("Height") }
            SpinBox { id: otioHeight; from: 1; to: 16384; value: 1080 }
            Label { text: qsTr("FPS numerator") }
            SpinBox { id: fpsNumerator; from: 1; to: 240000; value: 30 }
            Label { text: qsTr("FPS denominator") }
            SpinBox { id: fpsDenominator; from: 1; to: 1001; value: 1 }
        }
    }

    MessageDialog {
        id: repairDialog
        title: qsTr("Project Timing Needs Repair")
        text: qsTr("Some clips are not aligned to the project frame grid. Fixing them will save a new project without changing the original.")
        buttons: MessageDialog.Ok | MessageDialog.Cancel
        onAccepted: backend.confirmRepair(true)
        onRejected: backend.confirmRepair(false)
    }

    FileDialog {
        id: destinationDialog
        title: window.destinationTitle
        fileMode: FileDialog.SaveFile
        nameFilters: [qsTr("Shrimply projects (*.shrimp)")]
        currentFile: StandardPaths.writableLocation(StandardPaths.DocumentsLocation) + "/" + window.destinationName
        onAccepted: backend.chooseDestination(true, selectedFile)
        onRejected: backend.chooseDestination(false, "")
    }

    MessageDialog {
        id: warningDialog
        title: qsTr("OTIO imported with limitations")
        buttons: MessageDialog.Ok
        onAccepted: backend.acknowledgeWarnings()
    }

    Dialog {
        id: lockDialog
        property int pid: 0
        title: qsTr("Project is in use")
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.NoButton

        ColumnLayout {
            Label { text: qsTr("The project lock is held by another editor process (PID %1).").arg(lockDialog.pid) }
            RowLayout {
                Button { text: qsTr("Close"); onClicked: { lockDialog.close(); backend.resolveLock(0) } }
                Button { text: qsTr("Stop Other Editor"); onClicked: { lockDialog.close(); backend.resolveLock(2) } }
                Button { text: qsTr("Retry"); highlighted: true; onClicked: { lockDialog.close(); backend.resolveLock(1) } }
            }
        }
    }

    MessageDialog {
        id: errorDialog
        buttons: MessageDialog.Close
        onAccepted: Qt.quit()
    }

    MessageDialog {
        id: audioErrorDialog
        title: qsTr("Audio playback stopped")
        buttons: MessageDialog.Close
    }

    Shortcut { sequence: "Space"; enabled: backend.ready; onActivated: backend.togglePlaying() }
    Shortcut { sequence: "Left"; enabled: backend.ready; onActivated: backend.stepFrame(-1) }
    Shortcut { sequence: "Right"; enabled: backend.ready; onActivated: backend.stepFrame(1) }
}
