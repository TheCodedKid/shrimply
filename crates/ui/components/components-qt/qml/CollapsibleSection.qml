import QtQuick
import QtQuick.Layouts

Item {
    id: root
    property bool expanded: false
    property real spacing: 0
    default property alias content: content.data

    visible: expanded
    clip: true
    implicitHeight: expanded ? content.implicitHeight : 0
    Layout.preferredHeight: implicitHeight

    ColumnLayout {
        id: content
        width: parent.width
        spacing: root.spacing
    }
}
