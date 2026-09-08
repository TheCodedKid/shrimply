import QtQuick
import QtQuick.Controls

Button {
    id: root
    enum State { Idle, Indeterminate, Progress }
    property int progressState: ProgressButton.Idle
    property real progress: 0

    Rectangle {
        id: indicator
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        height: 2
        width: root.progressState === ProgressButton.Progress
            ? parent.width * Math.max(0, Math.min(1, root.progress))
            : parent.width * 0.25
        visible: root.progressState !== ProgressButton.Idle
        color: root.palette.highlight
        transform: Translate { id: movement }
        SequentialAnimation {
            running: root.progressState === ProgressButton.Indeterminate
            loops: Animation.Infinite
            NumberAnimation { target: movement; property: "x"; from: 0; to: root.width * 0.75; duration: 1000 }
            NumberAnimation { target: movement; property: "x"; from: root.width * 0.75; to: 0; duration: 1000 }
        }
    }
}
