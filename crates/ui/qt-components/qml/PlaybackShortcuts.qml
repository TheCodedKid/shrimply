import QtQuick

Item {
    id: root
    signal togglePlaying()
    signal stepPlaybackSpeedForward()
    focus: true
    Keys.onSpacePressed: function(event) { root.togglePlaying(); event.accepted = true }
    Keys.onPressed: function(event) {
        if (event.key === Qt.Key_L) {
            root.stepPlaybackSpeedForward()
            event.accepted = true
        }
    }
    TapHandler { onTapped: root.forceActiveFocus() }
}
