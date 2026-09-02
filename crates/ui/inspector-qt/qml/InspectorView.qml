pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.shrimply.components
import dev.shrimply.inspector

Item {
    id: root
    implicitWidth: backend.minimumWidth()
    signal error(string body)
    signal confirmation(string body)

    function refreshed(value) {
        const revision = backend.revision
        return revision >= 0 ? value : value
    }

    InspectorBackend {
        id: backend
        onShowError: function(body) { root.error(body) }
        onShowConfirmation: function(body) { root.confirmation(body) }
        onOpenPath: function(url) { Qt.openUrlExternally(url) }
    }

    Timer {
        interval: 50
        repeat: true
        running: true
        triggeredOnStart: true
        onTriggered: {
            if (backend.targetChangePending())
                root.forceActiveFocus(Qt.OtherFocusReason)
            backend.poll(inspectorScroll.contentItem.contentY)
        }
    }

    Label {
        anchors.centerIn: parent
        visible: !backend.ready
        text: qsTr("Loading inspector…")
        opacity: 0.65
    }

    ColumnLayout {
        anchors.fill: parent
        visible: backend.ready
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 16
            Layout.rightMargin: 16
            Layout.topMargin: 12
            Layout.bottomMargin: 8
            spacing: 0

            ButtonGroup { id: categories }
            Repeater {
                model: {
                    const revision = backend.revision
                    return revision >= 0 ? backend.categoryCount() : 0
                }
                Button {
                    required property int index
                    Layout.fillWidth: true
                    checkable: true
                    ButtonGroup.group: categories
                    checked: index === backend.activeCategory
                    text: root.refreshed(backend.categoryLabel(index))
                    icon.name: root.refreshed(backend.categoryIcon(index))
                    display: AbstractButton.TextBesideIcon
                    onClicked: backend.activateCategory(index)
                }
            }
        }

        ScrollView {
            id: inspectorScroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: availableWidth

            Connections {
                target: backend
                function onDocumentRevisionChanged() {
                    Qt.callLater(function() {
                        inspectorScroll.contentItem.contentY = backend.scrollPosition
                        inspectorScroll.contentItem.returnToBounds()
                    })
                }
            }

            ColumnLayout {
                x: 16
                y: 4
                width: parent.width - 32
                spacing: 8

                Repeater {
                    model: {
                        const revision = backend.revision
                        return revision >= 0 ? backend.itemCount(backend.activeCategory) : 0
                    }

                    Loader {
                        id: itemLoader
                        required property int index
                        readonly property int categoryIndex: backend.activeCategory
                        readonly property int revision: backend.revision
                        readonly property bool card: root.refreshed(
                            backend.itemIsCard(categoryIndex, index))
                        readonly property string identity: card ? root.refreshed(
                            backend.itemIdentity(categoryIndex, index)) : ""
                        Layout.fillWidth: true
                        sourceComponent: card ? cardComponent : flatComponent

                        Component {
                            id: flatComponent
                            Loader {
                                width: itemLoader.width
                                sourceComponent: controlList
                                onLoaded: {
                                    item.categoryIndex = Qt.binding(function() {
                                        return itemLoader.categoryIndex
                                    })
                                    item.itemIndex = Qt.binding(function() {
                                        return itemLoader.index
                                    })
                                }
                            }
                        }

                        Component {
                            id: cardComponent
                            InspectorCard {
                                id: card
                                readonly property string identity: itemLoader.identity
                                property string loadedBodyIdentity: ""
                                autoToggleExpansion: false
                                accented: root.refreshed(backend.itemFocused(
                                    itemLoader.categoryIndex, itemLoader.index))
                                title: root.refreshed(backend.itemTitle(
                                    itemLoader.categoryIndex, itemLoader.index))
                                resetVisible: root.refreshed(backend.itemResetAvailable(
                                    itemLoader.categoryIndex, itemLoader.index))
                                expanded: root.refreshed(backend.itemExpanded(
                                    itemLoader.categoryIndex, itemLoader.index))
                                onExpandedRequested: function(expanded) {
                                    backend.setItemExpanded(itemLoader.categoryIndex,
                                        itemLoader.index, expanded)
                                }
                                onExpandedChanged: if (expanded)
                                    loadedBodyIdentity = identity
                                onIdentityChanged: {
                                    loadedBodyIdentity = ""
                                    const pendingIdentity = identity
                                    Qt.callLater(function() {
                                        if (card.identity === pendingIdentity && card.expanded)
                                            card.loadedBodyIdentity = pendingIdentity
                                    })
                                }
                                onResetRequested: backend.resetItem(
                                    itemLoader.categoryIndex, itemLoader.index)
                                onFocusRequested: focusPreview()

                                function focusPreview() {
                                    if (backend.itemFocusAvailable(
                                            itemLoader.categoryIndex, itemLoader.index))
                                        backend.focusItem(
                                            itemLoader.categoryIndex, itemLoader.index)
                                }

                                beforeReset: RowLayout {
                                    spacing: 4
                                    Switch {
                                        visible: root.refreshed(backend.itemHasToggle(
                                            itemLoader.categoryIndex, itemLoader.index))
                                        checked: root.refreshed(backend.itemToggleActive(
                                            itemLoader.categoryIndex, itemLoader.index))
                                        ToolTip.visible: hovered
                                        ToolTip.text: root.refreshed(backend.itemToggleTooltip(
                                            itemLoader.categoryIndex, itemLoader.index))
                                        onToggled: backend.setItemToggle(
                                            itemLoader.categoryIndex, itemLoader.index, checked)
                                    }
                                    ToolButton {
                                        visible: root.refreshed(backend.itemHasButtonToggle(
                                            itemLoader.categoryIndex, itemLoader.index))
                                        checkable: true
                                        checked: root.refreshed(backend.itemButtonToggleActive(
                                            itemLoader.categoryIndex, itemLoader.index))
                                        icon.name: root.refreshed(backend.itemButtonToggleIcon(
                                            itemLoader.categoryIndex, itemLoader.index))
                                        ToolTip.visible: hovered
                                        ToolTip.text: root.refreshed(
                                            backend.itemButtonToggleTooltip(
                                                itemLoader.categoryIndex, itemLoader.index))
                                        onToggled: backend.setItemButtonToggle(
                                            itemLoader.categoryIndex, itemLoader.index, checked)
                                    }
                                }

                                afterReset: RowLayout {
                                    spacing: 4
                                    Repeater {
                                        model: root.refreshed(backend.itemActionCount(
                                            itemLoader.categoryIndex, itemLoader.index))
                                        ToolButton {
                                            required property int index
                                            enabled: root.refreshed(
                                                backend.itemActionSensitive(
                                                    itemLoader.categoryIndex,
                                                    itemLoader.index, index))
                                            icon.name: root.refreshed(backend.itemActionIcon(
                                                itemLoader.categoryIndex,
                                                itemLoader.index, index))
                                            ToolTip.visible: hovered
                                            ToolTip.text: root.refreshed(
                                                backend.itemActionTooltip(
                                                    itemLoader.categoryIndex,
                                                    itemLoader.index, index))
                                            onClicked: backend.triggerItemAction(
                                                itemLoader.categoryIndex, itemLoader.index, index)
                                        }
                                    }
                                }

                                Loader {
                                    Layout.fillWidth: true
                                    sourceComponent: card.identity.length > 0
                                            && card.loadedBodyIdentity === card.identity
                                        ? controlList : undefined
                                    onLoaded: {
                                        item.categoryIndex = Qt.binding(function() {
                                            return itemLoader.categoryIndex
                                        })
                                        item.itemIndex = Qt.binding(function() {
                                            return itemLoader.index
                                        })
                                    }
                                }
                                Component.onCompleted: if (expanded)
                                    loadedBodyIdentity = identity
                            }
                        }
                    }
                }

                Item { implicitHeight: 8 }
            }
        }
    }

    Component {
        id: controlList
        ColumnLayout {
            id: controlColumn
            property int categoryIndex: 0
            property int itemIndex: 0
            spacing: 8

            Repeater {
                model: root.refreshed(
                    backend.controlCount(parent.categoryIndex, parent.itemIndex))
                Loader {
                    id: controlLoader
                    required property int index
                    readonly property int categoryIndex: controlColumn.categoryIndex
                    readonly property int itemIndex: controlColumn.itemIndex
                    readonly property int kind: root.refreshed(backend.controlKind(
                        categoryIndex, itemIndex, index))
                    readonly property string label: root.refreshed(backend.controlLabel(
                        categoryIndex, itemIndex, index))
                    readonly property string value: {
                        const revision = kind === 18
                            ? backend.documentRevision + backend.cacheRevision
                            : backend.revision
                        return revision >= 0 ? backend.controlValue(
                            categoryIndex, itemIndex, index) : ""
                    }
                    readonly property bool editable: root.refreshed(backend.controlEditable(
                        categoryIndex, itemIndex, index))
                    readonly property bool sensitive: {
                        const revision = kind === 18 || kind === 19
                            ? backend.documentRevision + backend.cacheRevision
                            : backend.revision
                        return revision >= 0 && backend.controlSensitive(
                            categoryIndex, itemIndex, index)
                    }
                    visible: root.refreshed(
                        backend.controlVisible(categoryIndex, itemIndex, index))
                    enabled: kind === 5 || kind === 7 || kind === 8
                        || kind === 15 || kind >= 23
                            ? sensitive : editable && sensitive
                    Layout.fillWidth: true
                    sourceComponent: kind === 0 ? booleanControl
                        : kind === 1 ? numberControl
                        : kind === 2 ? fractionControl
                        : kind === 3 ? textControl
                        : kind === 4 ? multilineControl
                        : kind === 5 ? readOnlyControl
                        : kind === 6 ? colorControl
                        : kind === 7 || kind === 8 ? selectorControl
                        : kind === 9 ? vector2Control
                        : kind === 10 ? vector3Control
                        : kind === 11 ? layeredNumberControl
                        : kind === 12 ? layeredVector2Control
                        : kind === 13 ? layeredVector3Control
                        : kind === 14 ? projectSettingsControl
                        : kind === 15 ? performanceControl
                        : kind === 16 ? layeredBooleanControl
                        : kind === 17 ? layeredSelectorControl
                        : kind === 18 ? audioCacheControl
                        : kind === 19 ? selectorControl
                        : kind === 20 ? audioModifierMenuControl
                        : kind === 21 ? ttsEditorControl
                        : kind === 22 ? beatDetectionControl
                        : kind === 23 ? infoHeadingControl
                        : kind === 24 ? infoArtworkControl
                        : kind === 25 ? fileLocationControl
                        : kind === 26 ? infoLoadingControl
                        : actionControl

                    function component(componentIndex) {
                        const revision = kind === 18
                            ? backend.documentRevision + backend.cacheRevision
                            : backend.revision
                        return revision >= 0 ? backend.controlComponent(
                            categoryIndex, itemIndex, index, componentIndex) : 0
                    }
                    function componentText(componentIndex) {
                        return root.refreshed(backend.controlComponentText(
                            categoryIndex, itemIndex, index, componentIndex))
                    }
                    function tooltip() {
                        const revision = kind === 18
                            ? backend.documentRevision + backend.cacheRevision
                            : backend.revision
                        return revision >= 0 ? backend.controlTooltip(
                            categoryIndex, itemIndex, index) : ""
                    }
                    function minimum() {
                        return root.refreshed(backend.controlMinimum(
                            categoryIndex, itemIndex, index))
                    }
                    function maximum() {
                        return root.refreshed(backend.controlMaximum(
                            categoryIndex, itemIndex, index))
                    }
                    function dragStep() {
                        return root.refreshed(backend.controlDragStep(
                            categoryIndex, itemIndex, index))
                    }
                    function digits() {
                        return root.refreshed(backend.controlDigits(
                            categoryIndex, itemIndex, index))
                    }
                    function unit() {
                        return root.refreshed(backend.controlUnit(
                            categoryIndex, itemIndex, index))
                    }
                    function prefix(componentIndex) {
                        return root.refreshed(backend.controlPrefix(
                            categoryIndex, itemIndex, index, componentIndex))
                    }
                    function locked() {
                        return root.refreshed(backend.controlLock(
                            categoryIndex, itemIndex, index))
                    }
                    function edit(next) {
                        backend.setControlValue(categoryIndex, itemIndex, index, String(next))
                    }
                    function commit() {
                        backend.commitControl(categoryIndex, itemIndex, index)
                    }

                    Component {
                        id: booleanControl
                        SwitchRow {
                            label: controlLoader.label
                            subtitle: root.refreshed(backend.controlSubtitle(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            tooltip: controlLoader.tooltip()
                            active: controlLoader.value === "true"
                            onToggled: function(active) { controlLoader.edit(active) }
                        }
                    }
                    Component {
                        id: beatDetectionControl
                        SwitchRow {
                            id: beatRow
                            property int polls: 0
                            property bool observedBusy: false
                            label: controlLoader.label
                            subtitle: root.refreshed(backend.controlSubtitle(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            active: controlLoader.value === "true"
                            onActiveChanged: if (!active) {
                                busy = false
                                observedBusy = false
                                polls = 0
                            }
                            onToggled: function(active) {
                                polls = 0
                                observedBusy = false
                                controlLoader.edit(active)
                            }
                            Timer {
                                interval: 33
                                repeat: true
                                running: beatRow.active
                                    && (beatRow.busy || beatRow.polls < 60)
                                onTriggered: {
                                    const loading = backend.controlBusy(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex, controlLoader.index)
                                    beatRow.busy = loading
                                    beatRow.observedBusy = beatRow.observedBusy || loading
                                    beatRow.polls += 1
                                    if (beatRow.observedBusy && !loading)
                                        beatRow.polls = 60
                                }
                            }
                        }
                    }
                    Component {
                        id: ttsEditorControl
                        TtsEditor {
                            audioId: controlLoader.value
                            documentRevision: backend.documentRevision
                            onError: function(message) { root.error(message) }
                            onOpenPath: function(url) { Qt.openUrlExternally(url) }
                        }
                    }
                    Component {
                        id: numberControl
                        ControlRow {
                            label: controlLoader.label
                            NumberPicker {
                                value: Number(controlLoader.value)
                                minimum: controlLoader.minimum()
                                maximum: controlLoader.maximum()
                                dragStep: controlLoader.dragStep()
                                digits: controlLoader.digits()
                                unitName: controlLoader.unit()
                                widthCharacters: root.refreshed(
                                    backend.controlWidthCharacters(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex,
                                        controlLoader.index))
                                onEdited: function(value) { controlLoader.edit(value) }
                                onCommitted: function(value) { controlLoader.commit() }
                            }
                        }
                    }
                    Component {
                        id: fractionControl
                        ControlRow {
                            label: controlLoader.label
                            NumberPicker {
                                value: Number(controlLoader.value)
                                fractionNumerator: controlLoader.componentText(0)
                                fractionDenominator: controlLoader.componentText(1)
                                minimum: controlLoader.minimum()
                                maximum: controlLoader.maximum()
                                dragStep: controlLoader.dragStep()
                                digits: controlLoader.digits()
                                unitName: controlLoader.unit()
                                widthCharacters: root.refreshed(
                                    backend.controlWidthCharacters(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex,
                                        controlLoader.index))
                                onFractionEdited: function(numerator, denominator) {
                                    backend.setControlFraction(controlLoader.categoryIndex,
                                        controlLoader.itemIndex, controlLoader.index,
                                        numerator, denominator)
                                }
                                onCommitted: function(value) { controlLoader.commit() }
                            }
                        }
                    }
                    Component {
                        id: textControl
                        ControlRow {
                            label: controlLoader.label
                            SingleLineTextInput {
                                value: controlLoader.value
                                onEdited: function(value) { controlLoader.edit(value) }
                                onCommitted: function(value) { controlLoader.commit() }
                            }
                        }
                    }
                    Component {
                        id: multilineControl
                        ControlRow {
                            label: controlLoader.label
                            HoverHandler { id: multilineHover }
                            ToolTip.visible: multilineHover.hovered
                                && controlLoader.tooltip().length > 0
                            ToolTip.text: controlLoader.tooltip()
                            MultilineTextInput {
                                value: controlLoader.value
                                onEdited: function(value) { controlLoader.edit(value) }
                                onCommitted: function(value) { controlLoader.commit() }
                            }
                        }
                    }
                    Component {
                        id: readOnlyControl
                        ControlRow {
                            label: controlLoader.label
                            ReadOnlyField { text: controlLoader.value }
                        }
                    }
                    Component {
                        id: infoHeadingControl
                        Label {
                            text: controlLoader.label
                            font.bold: true
                            Layout.fillWidth: true
                            Layout.topMargin: 8
                            wrapMode: Text.Wrap
                        }
                    }
                    Component {
                        id: infoArtworkControl
                        ColumnLayout {
                            spacing: 6
                            Label {
                                text: controlLoader.label
                                font.bold: true
                                Layout.fillWidth: true
                            }
                            Image {
                                source: controlLoader.value
                                asynchronous: true
                                cache: true
                                fillMode: Image.PreserveAspectFit
                                sourceSize.height: 220
                                Layout.fillWidth: true
                                Layout.preferredHeight: 220
                            }
                        }
                    }
                    Component {
                        id: fileLocationControl
                        ControlRow {
                            label: controlLoader.label
                            RowLayout {
                                spacing: 6
                                ReadOnlyField {
                                    text: controlLoader.value
                                    Layout.fillWidth: true
                                }
                                ToolButton {
                                    icon.name: "folder-open-symbolic"
                                    ToolTip.visible: hovered
                                    ToolTip.text: controlLoader.tooltip()
                                    onClicked: backend.showControlPath(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex,
                                        controlLoader.index)
                                }
                            }
                        }
                    }
                    Component {
                        id: infoLoadingControl
                        ControlRow {
                            label: controlLoader.label
                            RowLayout {
                                spacing: 6
                                BusyIndicator {
                                    running: true
                                    implicitWidth: 18
                                    implicitHeight: 18
                                }
                                Label { text: controlLoader.value }
                            }
                        }
                    }
                    Component {
                        id: colorControl
                        ControlRow {
                            label: controlLoader.label
                            ColorPicker {
                                title: controlLoader.tooltip().length > 0
                                    ? controlLoader.tooltip() : controlLoader.label
                                color: Qt.rgba(
                                    controlLoader.component(0) / 255,
                                    controlLoader.component(1) / 255,
                                    controlLoader.component(2) / 255,
                                    controlLoader.component(3) / 255)
                                withAlpha: root.refreshed(backend.controlWithAlpha(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex,
                                    controlLoader.index))
                                onSelected: function(color) {
                                    backend.setControlColor(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex,
                                        controlLoader.index,
                                        color.r, color.g, color.b, color.a)
                                }
                                onScreenColorFailed: function(message) { root.error(message) }
                            }
                        }
                    }
                    Component {
                        id: selectorControl
                        ControlRow {
                            id: selectorRow
                            readonly property var values: root.refreshed(
                                backend.controlChoiceValues(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex,
                                    controlLoader.index))
                            readonly property var labels: root.refreshed(
                                backend.controlChoiceLabels(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex,
                                    controlLoader.index))
                            readonly property var icons: root.refreshed(
                                backend.controlChoiceIcons(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex,
                                    controlLoader.index))
                            label: controlLoader.label
                            HoverHandler { id: selectorHover }
                            ToolTip.visible: selectorHover.hovered
                                && controlLoader.tooltip().length > 0
                            ToolTip.text: controlLoader.tooltip()
                            RowLayout {
                                spacing: 6
                                Loader {
                                    Layout.fillWidth: true
                                    sourceComponent: selectorRow.icons.length > 0
                                        ? buttonSelector : dropdownSelector
                                }
                                BusyIndicator {
                                    visible: running
                                    running: backend.controlBusy(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex,
                                        controlLoader.index)
                                    implicitWidth: 18
                                    implicitHeight: 18
                                }
                            }
                            Component {
                                id: dropdownSelector
                                Dropdown {
                                    enabled: controlLoader.editable && controlLoader.sensitive
                                    value: controlLoader.value
                                    values: selectorRow.values
                                    labels: selectorRow.labels
                                    onSelected: function(value) {
                                        controlLoader.edit(value)
                                        controlLoader.commit()
                                    }
                                }
                            }
                            Component {
                                id: buttonSelector
                                ButtonSelector {
                                    enabled: controlLoader.editable && controlLoader.sensitive
                                    value: controlLoader.value
                                    values: selectorRow.values
                                    labels: selectorRow.labels
                                    icons: selectorRow.icons
                                    onSelected: function(value) {
                                        controlLoader.edit(value)
                                        controlLoader.commit()
                                    }
                                }
                            }
                        }
                    }
                    Component {
                        id: vector2Control
                        ControlRow {
                            label: controlLoader.label
                            Number2Picker {
                                first: controlLoader.component(0)
                                second: controlLoader.component(1)
                                minimum: controlLoader.minimum()
                                maximum: controlLoader.maximum()
                                dragStep: controlLoader.dragStep()
                                digits: controlLoader.digits()
                                unitName: controlLoader.unit()
                                firstPrefix: controlLoader.prefix(0)
                                secondPrefix: controlLoader.prefix(1)
                                enableLock: controlLoader.locked()
                                onValuesEdited: function(first, second, component) {
                                    backend.setControlComponents(controlLoader.categoryIndex,
                                        controlLoader.itemIndex, controlLoader.index,
                                        first, second, 0, component)
                                }
                                onFirstCommitted: function(value) { controlLoader.commit() }
                                onSecondCommitted: function(value) { controlLoader.commit() }
                            }
                        }
                    }
                    Component {
                        id: vector3Control
                        ControlRow {
                            label: controlLoader.label
                            Number3Picker {
                                first: controlLoader.component(0)
                                second: controlLoader.component(1)
                                third: controlLoader.component(2)
                                minimum: controlLoader.minimum()
                                maximum: controlLoader.maximum()
                                dragStep: controlLoader.dragStep()
                                digits: controlLoader.digits()
                                unitName: controlLoader.unit()
                                prefixes: [
                                    controlLoader.prefix(0),
                                    controlLoader.prefix(1),
                                    controlLoader.prefix(2)
                                ]
                                enableLock: controlLoader.locked()
                                onValuesEdited: function(first, second, third, component) {
                                    backend.setControlComponents(controlLoader.categoryIndex,
                                        controlLoader.itemIndex, controlLoader.index,
                                        first, second, third, component)
                                }
                                onCommitted: function(axis, value) { controlLoader.commit() }
                            }
                        }
                    }
                    Component {
                        id: layeredNumberControl
                        InspectorGraphProperty {
                            id: propertyGraph
                            readonly property int graphDocumentRevision: backend.documentRevision
                            readonly property int graphPlayheadRevision:
                                backend.playheadRevision
                            readonly property var expressionResult: {
                                const documentRevision = backend.documentRevision
                                const expressionRevision = backend.expressionRevision
                                const playheadRevision = backend.playheadRevision
                                return documentRevision + expressionRevision
                                        + playheadRevision >= 0
                                    ? backend.controlExpressionResult(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex, controlLoader.index)
                                    : []
                            }
                            label: controlLoader.label
                            initialGraphValue: Number(controlLoader.value)
                            keyframes: root.refreshed(backend.controlKeyframes(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            expression: root.refreshed(backend.controlExpression(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            expressionValue: root.refreshed(
                                backend.controlExpressionSource(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index))
                            expressionOutput: expressionResult.length > 0
                                ? expressionResult[0] : ""
                            expressionError: expressionResult.length > 1
                                ? expressionResult[1] : ""
                            externalClipboardMarker: backend.keyframeClipboardMarker()
                            function updateGraphPlayhead() {
                                if (!propertyGraph.keyframes)
                                    return
                                const playhead = backend.controlGraphPlayhead(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                if (playhead.length === 2)
                                    propertyGraph.setGraphPlayhead(playhead[0], playhead[1])
                            }
                            function configureGraph() {
                                if (!propertyGraph.keyframes)
                                    return
                                const times = backend.controlGraphPointTimes(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                const values = backend.controlGraphPointValues(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                const segments = backend.controlGraphSegments(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                const timing = backend.controlGraphTiming(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                if (timing.length !== 6)
                                    return
                                propertyGraph.replaceRawGraphComponent(
                                    0, times, values, segments, Number(controlLoader.value))
                                propertyGraph.setGraphRange(
                                    timing[0], timing[1], timing[2], timing[3])
                                propertyGraph.setGraphFrameStep(timing[4], timing[5])
                                propertyGraph.setGraphSnapping(
                                    backend.keyframeSnappingEnabled(),
                                    backend.keyframeSnappingRadius())
                                propertyGraph.setGraphExternalClipboard(true)
                                propertyGraph.updateGraphPlayhead()
                            }
                            Component.onCompleted: configureGraph()
                            onGraphLoaded: configureGraph()
                            onGraphDocumentRevisionChanged: configureGraph()
                            onGraphPlayheadRevisionChanged: updateGraphPlayhead()
                            NumberPicker {
                                value: propertyGraph.graphValue
                                minimum: controlLoader.minimum()
                                maximum: controlLoader.maximum()
                                dragStep: controlLoader.dragStep()
                                digits: controlLoader.digits()
                                unitName: controlLoader.unit()
                                onEdited: function(value) { propertyGraph.editValue(value) }
                                onCommitted: function(value) { controlLoader.commit() }
                            }
                            onBaseValueEdited: function(value) { controlLoader.edit(value) }
                            onKeyframeValueEdited: function(value) { controlLoader.edit(value) }
                            onGraphPlaybackToggled: backend.toggleControlGraphPlayback()
                            onGraphPlayheadChanged: function(component, numerator, denominator) {
                                if (component !== 0)
                                    return
                                backend.seekControlGraph(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    numerator, denominator)
                            }
                            onGraphKeysMoved: function(component, oldTimes, times, values) {
                                if (component !== 0)
                                    return
                                backend.moveControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    oldTimes, times, values)
                            }
                            onGraphEditFinished: controlLoader.commit()
                            onGraphKeysDeleted: function(component, times) {
                                if (component !== 0)
                                    return
                                backend.deleteControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, times)
                            }
                            onGraphKeyAdded: function(component, numerator, denominator) {
                                if (component !== 0)
                                    return
                                backend.addControlGraphKey(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    numerator, denominator)
                            }
                            onGraphCopyRequested: function(component, times) {
                                if (component !== 0)
                                    return
                                if (backend.copyControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, times)) {
                                    propertyGraph.copyExternalClipboardMarker()
                                }
                            }
                            onGraphPasteRequested: function(component, numerator, denominator) {
                                if (component !== 0)
                                    return
                                backend.pasteControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    numerator, denominator)
                            }
                            onGraphInterpolationChanged: function(component, ownerId, interpolation) {
                                if (component !== 0)
                                    return
                                backend.setControlGraphInterpolation(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    ownerId, interpolation)
                            }
                            onKeyframesToggled: function(enabled) {
                                backend.setControlKeyframes(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, enabled)
                            }
                            onExpressionToggled: function(enabled) {
                                backend.setControlExpression(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, enabled)
                            }
                            onExpressionEdited: function(value) {
                                backend.setControlExpressionSource(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, value)
                            }
                            onExpressionCommitted: controlLoader.commit()
                        }
                    }
                    Component {
                        id: layeredBooleanControl
                        InspectorGraphProperty {
                            id: propertyGraph
                            readonly property int graphDocumentRevision: backend.documentRevision
                            readonly property int graphPlayheadRevision:
                                backend.playheadRevision
                            readonly property var expressionResult: {
                                const documentRevision = backend.documentRevision
                                const expressionRevision = backend.expressionRevision
                                const playheadRevision = backend.playheadRevision
                                return documentRevision + expressionRevision
                                        + playheadRevision >= 0
                                    ? backend.controlExpressionResult(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex, controlLoader.index)
                                    : []
                            }
                            label: controlLoader.label
                            initialGraphValue: controlLoader.value === "true" ? 1 : 0
                            keyframes: root.refreshed(backend.controlKeyframes(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            expression: root.refreshed(backend.controlExpression(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            expressionValue: root.refreshed(
                                backend.controlExpressionSource(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index))
                            expressionOutput: expressionResult.length > 0
                                ? expressionResult[0] : ""
                            expressionError: expressionResult.length > 1
                                ? expressionResult[1] : ""
                            externalClipboardMarker: backend.keyframeClipboardMarker()
                            function updateGraphPlayhead() {
                                if (!propertyGraph.keyframes)
                                    return
                                const playhead = backend.controlGraphPlayhead(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                if (playhead.length === 2)
                                    propertyGraph.setGraphPlayhead(playhead[0], playhead[1])
                            }
                            function configureGraph() {
                                if (!propertyGraph.keyframes)
                                    return
                                const times = backend.controlGraphPointTimes(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                const values = backend.controlGraphPointValues(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                const timing = backend.controlGraphTiming(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                if (timing.length !== 6)
                                    return
                                propertyGraph.replaceStepGraphComponent(0, times, values)
                                propertyGraph.setGraphRange(
                                    timing[0], timing[1], timing[2], timing[3])
                                propertyGraph.setGraphFrameStep(timing[4], timing[5])
                                propertyGraph.setGraphSnapping(
                                    backend.keyframeSnappingEnabled(),
                                    backend.keyframeSnappingRadius())
                                propertyGraph.setGraphExternalClipboard(true)
                                propertyGraph.updateGraphPlayhead()
                            }
                            Component.onCompleted: configureGraph()
                            onGraphLoaded: configureGraph()
                            onGraphDocumentRevisionChanged: configureGraph()
                            onGraphPlayheadRevisionChanged: updateGraphPlayhead()
                            SwitchRow {
                                label: ""
                                active: propertyGraph.graphValue >= 0.5
                                onToggled: function(active) {
                                    propertyGraph.editValue(active ? 1 : 0)
                                    controlLoader.commit()
                                }
                            }
                            onBaseValueEdited: function(value) {
                                controlLoader.edit(value >= 0.5)
                            }
                            onKeyframeValueEdited: function(value) {
                                controlLoader.edit(value >= 0.5)
                            }
                            onGraphPlaybackToggled: backend.toggleControlGraphPlayback()
                            onGraphPlayheadChanged: function(component, numerator, denominator) {
                                if (component !== 0)
                                    return
                                backend.seekControlGraph(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    numerator, denominator)
                            }
                            onGraphKeysMoved: function(component, oldTimes, times, values) {
                                if (component !== 0)
                                    return
                                if (times.length === 0)
                                    return
                                const canonicalTimes = backend.moveControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    oldTimes, times, values)
                                if (canonicalTimes.length === times.length)
                                    propertyGraph.reconcileStepGraphMoves(
                                        component, oldTimes, times, canonicalTimes)
                                else
                                    propertyGraph.rollbackStepGraphMoves(
                                        component, oldTimes, times)
                            }
                            onGraphEditFinished: controlLoader.commit()
                            onGraphKeysDeleted: function(component, times) {
                                if (component !== 0)
                                    return
                                backend.deleteControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, times)
                            }
                            onGraphKeyAdded: function(component, numerator, denominator) {
                                if (component !== 0)
                                    return
                                backend.addControlGraphKey(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    numerator, denominator)
                            }
                            onGraphCopyRequested: function(component, times) {
                                if (component !== 0)
                                    return
                                if (backend.copyControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, times)) {
                                    propertyGraph.copyExternalClipboardMarker()
                                }
                            }
                            onGraphPasteRequested: function(component, numerator, denominator) {
                                if (component !== 0)
                                    return
                                backend.pasteControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    numerator, denominator)
                            }
                            onKeyframesToggled: function(enabled) {
                                backend.setControlKeyframes(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, enabled)
                            }
                            onExpressionToggled: function(enabled) {
                                backend.setControlExpression(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, enabled)
                            }
                            onExpressionEdited: function(value) {
                                backend.setControlExpressionSource(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, value)
                            }
                            onExpressionCommitted: controlLoader.commit()
                        }
                    }
                    Component {
                        id: layeredSelectorControl
                        InspectorGraphProperty {
                            id: propertyGraph
                            readonly property int graphDocumentRevision: backend.documentRevision
                            readonly property int graphPlayheadRevision:
                                backend.playheadRevision
                            readonly property var expressionResult: {
                                const documentRevision = backend.documentRevision
                                const expressionRevision = backend.expressionRevision
                                const playheadRevision = backend.playheadRevision
                                return documentRevision + expressionRevision
                                        + playheadRevision >= 0
                                    ? backend.controlExpressionResult(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex, controlLoader.index)
                                    : []
                            }
                            readonly property var values: root.refreshed(
                                backend.controlChoiceValues(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index))
                            readonly property int selectedIndex: Math.max(0,
                                values.indexOf(controlLoader.value))
                            label: controlLoader.label
                            initialGraphValue: selectedIndex
                            keyframes: root.refreshed(backend.controlKeyframes(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            expression: root.refreshed(backend.controlExpression(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            expressionValue: root.refreshed(
                                backend.controlExpressionSource(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index))
                            expressionOutput: expressionResult.length > 0
                                ? expressionResult[0] : ""
                            expressionError: expressionResult.length > 1
                                ? expressionResult[1] : ""
                            externalClipboardMarker: backend.keyframeClipboardMarker()
                            function updateGraphPlayhead() {
                                if (!propertyGraph.keyframes)
                                    return
                                const playhead = backend.controlGraphPlayhead(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                if (playhead.length === 2)
                                    propertyGraph.setGraphPlayhead(playhead[0], playhead[1])
                            }
                            function configureGraph() {
                                if (!propertyGraph.keyframes)
                                    return
                                const times = backend.controlGraphPointTimes(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                const pointValues = backend.controlGraphPointValues(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                const timing = backend.controlGraphTiming(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                                if (timing.length !== 6)
                                    return
                                propertyGraph.replaceStepGraphComponent(0, times, pointValues)
                                propertyGraph.setGraphRange(
                                    timing[0], timing[1], timing[2], timing[3])
                                propertyGraph.setGraphFrameStep(timing[4], timing[5])
                                propertyGraph.setGraphSnapping(
                                    backend.keyframeSnappingEnabled(),
                                    backend.keyframeSnappingRadius())
                                propertyGraph.setGraphExternalClipboard(true)
                                propertyGraph.updateGraphPlayhead()
                            }
                            Component.onCompleted: configureGraph()
                            onGraphLoaded: configureGraph()
                            onGraphDocumentRevisionChanged: configureGraph()
                            onGraphPlayheadRevisionChanged: updateGraphPlayhead()
                            Dropdown {
                                value: propertyGraph.values[Math.max(0, Math.min(
                                    propertyGraph.values.length - 1,
                                    Math.round(propertyGraph.graphValue)))] || ""
                                values: propertyGraph.values
                                labels: root.refreshed(backend.controlChoiceLabels(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index))
                                onSelected: function(value) {
                                    propertyGraph.editValue(propertyGraph.values.indexOf(value))
                                    controlLoader.commit()
                                }
                            }
                            onBaseValueEdited: function(value) {
                                const index = Math.max(0, Math.min(values.length - 1,
                                    Math.round(value)))
                                if (values.length > 0)
                                    controlLoader.edit(values[index])
                            }
                            onKeyframeValueEdited: function(value) {
                                const index = Math.max(0, Math.min(values.length - 1,
                                    Math.round(value)))
                                if (values.length > 0)
                                    controlLoader.edit(values[index])
                            }
                            onGraphPlaybackToggled: backend.toggleControlGraphPlayback()
                            onGraphPlayheadChanged: function(component, numerator, denominator) {
                                if (component !== 0)
                                    return
                                backend.seekControlGraph(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    numerator, denominator)
                            }
                            onGraphKeysMoved: function(component, oldTimes, times, pointValues) {
                                if (component !== 0)
                                    return
                                if (times.length === 0)
                                    return
                                const canonicalTimes = backend.moveControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    oldTimes, times, pointValues)
                                if (canonicalTimes.length === times.length)
                                    propertyGraph.reconcileStepGraphMoves(
                                        component, oldTimes, times, canonicalTimes)
                                else
                                    propertyGraph.rollbackStepGraphMoves(
                                        component, oldTimes, times)
                            }
                            onGraphEditFinished: controlLoader.commit()
                            onGraphKeysDeleted: function(component, times) {
                                if (component !== 0)
                                    return
                                backend.deleteControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, times)
                            }
                            onGraphKeyAdded: function(component, numerator, denominator) {
                                if (component !== 0)
                                    return
                                backend.addControlGraphKey(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    numerator, denominator)
                            }
                            onGraphCopyRequested: function(component, times) {
                                if (component !== 0)
                                    return
                                if (backend.copyControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, times)) {
                                    propertyGraph.copyExternalClipboardMarker()
                                }
                            }
                            onGraphPasteRequested: function(component, numerator, denominator) {
                                if (component !== 0)
                                    return
                                backend.pasteControlGraphKeys(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    numerator, denominator)
                            }
                            onKeyframesToggled: function(enabled) {
                                backend.setControlKeyframes(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, enabled)
                            }
                            onExpressionToggled: function(enabled) {
                                backend.setControlExpression(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, enabled)
                            }
                            onExpressionEdited: function(value) {
                                backend.setControlExpressionSource(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, value)
                            }
                            onExpressionCommitted: controlLoader.commit()
                        }
                    }
                    Component {
                        id: layeredVector2Control
                        InspectorPairGraphProperty {
                            label: controlLoader.label
                            initialGraphValue: controlLoader.component(0)
                            initialSecondValue: controlLoader.component(1)
                            keyframes: root.refreshed(backend.controlKeyframes(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            expression: root.refreshed(backend.controlExpression(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            expressionValue: root.refreshed(
                                backend.controlExpressionSource(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index))
                            minimum: controlLoader.minimum()
                            maximum: controlLoader.maximum()
                            dragStep: controlLoader.dragStep()
                            digits: controlLoader.digits()
                            unitName: controlLoader.unit()
                            enableLock: controlLoader.locked()
                            onBasePairEdited: function(first, second) {
                                backend.setControlComponents(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index,
                                    first, second, 0, 0)
                            }
                            onKeyframesToggled: function(enabled) {
                                backend.setControlKeyframes(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, enabled)
                            }
                            onExpressionToggled: function(enabled) {
                                backend.setControlExpression(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, enabled)
                            }
                            onExpressionEdited: function(value) {
                                backend.setControlExpressionSource(controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index, value)
                            }
                            onExpressionCommitted: controlLoader.commit()
                        }
                    }
                    Component {
                        id: actionControl
                        ControlRow {
                            label: controlLoader.label
                            subtitle: root.refreshed(backend.controlSubtitle(
                                controlLoader.categoryIndex,
                                controlLoader.itemIndex, controlLoader.index))
                            ProgressButton {
                                text: controlLoader.value
                                progressState: backend.controlBusy(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex,
                                    controlLoader.index)
                                        ? ProgressButton.Indeterminate : ProgressButton.Idle
                                ToolTip.visible: hovered && controlLoader.tooltip().length > 0
                                ToolTip.text: controlLoader.tooltip()
                                onClicked: backend.triggerControlAction(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                            }
                        }
                    }
                    Component {
                        id: audioCacheControl
                        ColumnLayout {
                            SystemPalette { id: systemPalette }
                            Layout.fillWidth: true
                            ProgressBar {
                                readonly property real progress: controlLoader.component(0)
                                visible: controlLoader.value === "Baking…"
                                indeterminate: progress < 0
                                value: Math.max(0, progress)
                                Layout.fillWidth: true
                            }
                            Button {
                                readonly property bool cancelHovered:
                                    controlLoader.value === "Baking…" && hovered
                                highlighted: controlLoader.value !== "Baking…"
                                text: controlLoader.value === "Baking…" && hovered
                                    ? qsTr("Cancel") : controlLoader.value
                                palette.button: cancelHovered
                                    ? backend.destructiveBackground() : systemPalette.button
                                palette.buttonText: cancelHovered
                                    ? backend.destructiveForeground() : systemPalette.buttonText
                                ToolTip.visible: hovered && controlLoader.tooltip().length > 0
                                ToolTip.text: controlLoader.tooltip()
                                Layout.alignment: Qt.AlignRight
                                onClicked: backend.triggerControlAction(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index)
                            }
                        }
                    }
                    Component {
                        id: audioModifierMenuControl
                        RowLayout {
                            Layout.fillWidth: true
                            Item { Layout.fillWidth: true }
                            ModifierMenuButton {
                                values: root.refreshed(backend.controlChoiceValues(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index))
                                labels: root.refreshed(backend.controlChoiceLabels(
                                    controlLoader.categoryIndex,
                                    controlLoader.itemIndex, controlLoader.index))
                                searchTerms: root.refreshed(
                                    backend.controlChoiceSearchTerms(
                                        controlLoader.categoryIndex,
                                        controlLoader.itemIndex, controlLoader.index))
                                onSelected: function(value) { controlLoader.edit(value) }
                            }
                            Button {
                                visible: controlLoader.value === "true"
                                flat: true
                                icon.name: "edit-paste-symbolic"
                                text: qsTr("Paste Modifier")
                                display: AbstractButton.IconOnly
                                ToolTip.visible: hovered
                                ToolTip.text: text
                                onClicked: controlLoader.edit("__paste__")
                            }
                            Item { Layout.fillWidth: true }
                        }
                    }
                    Component {
                        id: layeredVector3Control
                        Loader { sourceComponent: vector3Control }
                    }
                    Component {
                        id: projectSettingsControl
                        ProjectSettingsSelector {
                            initialWidth: controlLoader.component(0)
                            initialHeight: controlLoader.component(1)
                            initialFpsNumerator: controlLoader.componentText(2)
                            initialFpsDenominator: controlLoader.componentText(3)
                            onApplyRequested: function(width, height, numerator, denominator) {
                                backend.applyProjectSettings(width, height, numerator, denominator)
                            }
                        }
                    }
                    Component {
                        id: performanceControl
                        LivePerformance { Layout.fillWidth: true }
                    }
                }
            }
        }
    }
}
