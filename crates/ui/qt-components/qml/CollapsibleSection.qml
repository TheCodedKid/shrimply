import QtQuick
import QtQuick.Layouts

Item {
    id: root
    property bool expanded: false
    property real spacing: 0
    default property alias content: content.data

    property real reveal: expanded ? 1 : 0
    visible: reveal > 0
    clip: true
    implicitHeight: content.implicitHeight * reveal
    Layout.preferredHeight: implicitHeight

    Behavior on reveal {
        NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
    }

    ColumnLayout {
        id: content
        width: parent.width
        spacing: root.spacing
        opacity: root.reveal
    }
}
