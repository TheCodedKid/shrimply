import QtQuick
import dev.shrimply.components
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: root
    signal changed(int width, int height, int fpsNumerator, int fpsDenominator)
    spacing: 6

    ProjectSettingsBackend { id: backend }

    function notifyChanged() {
        root.changed(backend.width, backend.height, backend.fpsNumerator(), backend.fpsDenominator())
    }

    ControlRow {
        label: "Preset"
        ComboBox {
            currentIndex: backend.preset
            id: preset
            model: backend.presetCount
            textRole: "text"
            delegate: ItemDelegate {
                required property int index
                width: preset.width
                text: backend.presetLabel(index)
            }
            contentItem: Label { text: backend.presetLabel(backend.preset); verticalAlignment: Text.AlignVCenter }
            onActivated: { backend.selectPreset(currentIndex); root.notifyChanged() }
            Connections { target: backend; function onPresetChanged() { preset.currentIndex = backend.preset } }
        }
    }
    ControlRow {
        label: "Width"
        NumberPicker {
            value: backend.width
            minimum: 1
            maximum: 16384
            digits: 0
            dragStep: 1
            onEdited: function(value) { backend.setWidthValue(Math.round(value)); root.notifyChanged() }
        }
    }
    ControlRow {
        label: "Height"
        NumberPicker {
            value: backend.height
            minimum: 1
            maximum: 16384
            digits: 0
            dragStep: 1
            onEdited: function(value) { backend.setHeightValue(Math.round(value)); root.notifyChanged() }
        }
    }
    ControlRow {
        label: "Frame Rate"
        ComboBox {
            currentIndex: backend.frameRate
            id: fps
            model: backend.frameRateCount
            delegate: ItemDelegate {
                required property int index
                width: fps.width
                text: backend.frameRateLabel(index)
            }
            contentItem: Label { text: backend.frameRateLabel(backend.frameRate); verticalAlignment: Text.AlignVCenter }
            onActivated: { backend.setFrameRateValue(currentIndex); root.notifyChanged() }
            Connections { target: backend; function onFrameRateChanged() { fps.currentIndex = backend.frameRate } }
        }
    }
}
