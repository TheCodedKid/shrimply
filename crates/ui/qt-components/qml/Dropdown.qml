pragma ComponentBehavior: Bound

import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

Button {
    id: root
    property var values: []
    property var labels: []
    property string value: ""
    property bool enableSearch: backend.searchable(labels.length)
    property string searchPlaceholder: ComponentTranslations.text("Search")
    signal selected(string value)
    text: currentLabel()

    function currentLabel() {
        const index = backend.selectedIndex(values, value)
        return index >= 0 && index < labels.length ? labels[index] : ""
    }

    function synchronize() {
        popup.selectedIndex = backend.selectedIndex(values, value)
    }

    function choose(index) {
        const next = backend.valueAt(values, index)
        if (index < 0 || next.length === 0)
            return
        value = next
        selected(next)
    }

    SelectorBackend { id: backend }

    // Match the editor timeline menus: QQuickMenu's Wayland positioner uses
    // the parent item's bottom-left anchor with bottom-right gravity.  A wide
    // parent therefore places the popup at the parent's right edge; using a
    // one-pixel anchor makes that edge the field's left edge.
    Item {
        id: popupAnchor
        parent: root
        x: 0
        y: root.height
        width: 1
        height: 1
        visible: false
    }

    Component.onCompleted: synchronize()
    onValuesChanged: synchronize()
    onValueChanged: synchronize()
    onClicked: {
        if (popup.opened)
            popup.close()
        else
            popup.popup(popupAnchor, 0, 0)
    }

    contentItem: RowLayout {
        Label {
            text: root.text
            elide: Text.ElideRight
            Layout.fillWidth: true
        }
        Label { text: "⌄" }
    }

    SearchMenu {
        id: popup
        width: Math.max(root.width, 240)
        labels: root.labels
        selectedIndex: backend.selectedIndex(root.values, root.value)
        searchEnabled: root.enableSearch
        placeholderText: root.searchPlaceholder
        onActivated: function(index) { root.choose(index) }
        onClosed: root.forceActiveFocus(Qt.PopupFocusReason)
    }
}
