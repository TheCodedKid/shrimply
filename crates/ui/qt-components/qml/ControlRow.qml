import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

RowLayout {
    id: root
    property string label: ""
    default property alias control: holder.data
    spacing: 6

    Label {
        text: ComponentTranslations.text(root.label)
        opacity: 0.72
        horizontalAlignment: Text.AlignLeft
        verticalAlignment: Text.AlignVCenter
        Layout.preferredWidth: 156
    }

    Item {
        id: holder
        Layout.fillWidth: true
        implicitHeight: childrenRect.height
        onChildrenChanged: {
            for (let index = 0; index < children.length; ++index)
                children[index].width = Qt.binding(function() { return holder.width })
        }
    }
}
