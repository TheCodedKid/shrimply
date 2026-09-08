import QtQuick
import QtQuick.Controls
import dev.shrimply.components.native 1.0

ScrollView {
    id: root
    property string value: ""
    property int tabWidth: 2
    property bool showLineNumbers: false
    property var diagnosticProvider: null
    property var completionProvider: null
    property int diagnosticDebounce: 250
    property int completionDebounce: diagnosticDebounce
    property string diagnostic: ""
    property int diagnosticLine: -1
    property int diagnosticColumn: -1
    property bool dirty: false
    property bool synchronizing: false
    property bool applyingCompletion: false
    property int completionStart: 0
    property int completionEnd: 0
    signal edited(string value)
    signal committed(string value)
    implicitHeight: 86

    function forceEditorFocus() {
        editor.forceActiveFocus()
    }

    function currentText() {
        return editor.text
    }

    function synchronize(value) {
        if (editor.text === value)
            return
        completionTimer.stop()
        completionPopup.close()
        synchronizing = true
        editor.text = value
        synchronizing = false
        diagnosticTimer.restart()
    }

    function commit() {
        if (!dirty)
            return
        dirty = false
        root.committed(editor.text)
    }

    function refreshDiagnostic() {
        if (!diagnosticProvider) {
            diagnostic = ""
            diagnosticLine = -1
            diagnosticColumn = -1
            return
        }
        const result = diagnosticProvider(editor.text)
        diagnostic = result.length > 0 ? result[0] : ""
        diagnosticLine = result.length > 0
            ? result.length > 1 && result[1].length > 0 ? Number(result[1]) : 0
            : -1
        diagnosticColumn = result.length > 2 && result[2].length > 0
            ? Number(result[2]) : 0
    }

    function refreshCompletion(automatic) {
        if (!completionProvider || !editor.activeFocus
                || editor.selectionStart !== editor.selectionEnd) {
            completionPopup.close()
            return
        }
        const result = completionProvider(
            editor.text, editor.cursorPosition, automatic)
        if (result.length <= 2) {
            completionPopup.close()
            return
        }
        completionStart = Number(result[0])
        completionEnd = Number(result[1])
        const candidates = []
        for (let index = 2; index < result.length; ++index)
            candidates.push(result[index])
        completionPopup.candidates = candidates
        completionPopup.showAt(editor,
            editor.positionToRectangle(editor.cursorPosition))
    }

    function acceptCompletion(candidate) {
        completionTimer.stop()
        completionPopup.close()
        applyingCompletion = true
        editor.remove(completionStart, completionEnd)
        editor.insert(completionStart, candidate)
        editor.cursorPosition = completionStart + candidate.length
        applyingCompletion = false
        editor.forceActiveFocus()
    }

    function indentationBefore(position) {
        const lineStart = editor.text.lastIndexOf("\n", position - 1) + 1
        let end = lineStart
        while (end < position
                && (editor.text.charAt(end) === " " || editor.text.charAt(end) === "\t"))
            ++end
        return editor.text.substring(lineStart, end)
    }

    function insertIndentedNewline() {
        if (editor.selectionStart !== editor.selectionEnd)
            editor.remove(editor.selectionStart, editor.selectionEnd)
        const position = editor.cursorPosition
        const indentation = indentationBefore(position)
        const previous = position > 0 ? editor.text.charAt(position - 1) : ""
        const closing = previous === "(" ? ")"
            : previous === "[" ? "]" : previous === "{" ? "}" : ""
        const nested = closing.length > 0
        if (nested && editor.text.charAt(position) === closing) {
            const insertion = "\n" + indentation + " ".repeat(tabWidth)
                + "\n" + indentation
            editor.insert(position, insertion)
            editor.cursorPosition = position + 1 + indentation.length + tabWidth
        } else {
            const insertion = "\n" + indentation + (nested ? " ".repeat(tabWidth) : "")
            editor.insert(position, insertion)
        }
    }

    function toggleComments() {
        let first = editor.selectionStart
        let last = editor.selectionEnd
        first = editor.text.lastIndexOf("\n", Math.max(0, first - 1)) + 1
        if (last > first && editor.text.charAt(last - 1) === "\n")
            --last
        const after = editor.text.indexOf("\n", last)
        last = after < 0 ? editor.text.length : after
        const block = editor.text.substring(first, last)
        const lines = block.split("\n")
        let uncomment = true
        for (let index = 0; index < lines.length; ++index) {
            const trimmed = lines[index].trimStart()
            if (trimmed.length > 0 && !trimmed.startsWith("//")) {
                uncomment = false
                break
            }
        }
        for (let index = 0; index < lines.length; ++index) {
            const line = lines[index]
            const trimmed = line.trimStart()
            if (trimmed.length === 0)
                continue
            const indent = line.length - trimmed.length
            lines[index] = uncomment
                ? line.substring(0, indent)
                    + trimmed.substring(trimmed.startsWith("// ") ? 3 : 2)
                : line.substring(0, indent) + "// " + trimmed
        }
        editor.remove(first, last)
        editor.insert(first, lines.join("\n"))
        editor.select(first, first + lines.join("\n").length)
    }

    function insertPair(open) {
        const close = open === "(" ? ")" : open === "[" ? "]" : open === "{" ? "}"
            : open
        const start = editor.selectionStart
        const end = editor.selectionEnd
        if (start === end && (open === "\"" || open === "'" || open === "`")) {
            let precedingBackslashes = 0
            for (let index = start - 1; index >= 0
                    && editor.text.charAt(index) === "\\"; --index)
                ++precedingBackslashes
            if (precedingBackslashes % 2 === 1)
                return false
            if (editor.text.charAt(start) === open) {
                editor.cursorPosition = start + 1
                return true
            }
        }
        if (start !== end) {
            const selected = editor.selectedText
            editor.remove(start, end)
            editor.insert(start, open + selected + close)
            editor.select(start + 1, start + 1 + selected.length)
        } else {
            editor.insert(start, open + close)
            editor.cursorPosition = start + 1
        }
        return true
    }

    Component.onCompleted: refreshDiagnostic()
    Component.onDestruction: commit()

    onValueChanged: if (!editor.activeFocus && editor.text !== value) {
        synchronize(value)
        dirty = false
    }

    Timer {
        id: diagnosticTimer
        interval: root.diagnosticDebounce
        onTriggered: root.refreshDiagnostic()
    }
    Timer {
        id: completionTimer
        interval: root.completionDebounce
        onTriggered: root.refreshCompletion(true)
    }

    CompletionPopup {
        id: completionPopup
        onAccepted: function(candidate) { root.acceptCompletion(candidate) }
    }

    TextArea {
        id: editor
        text: root.value
        wrapMode: TextEdit.WrapAtWordBoundaryOrAnywhere
        selectByMouse: true
        persistentSelection: true
        leftPadding: root.showLineNumbers ? 48 : 8
        font.family: "monospace"
        onTextChanged: if (!root.synchronizing && root.value !== text) {
            root.dirty = true
            root.edited(text)
            diagnosticTimer.restart()
            if (!root.applyingCompletion) {
                completionPopup.close()
                completionTimer.restart()
            }
        }
        onCursorPositionChanged: if (!root.synchronizing
                && !root.applyingCompletion && activeFocus) {
            completionPopup.close()
            completionTimer.restart()
        }
        onSelectionStartChanged: if (selectionStart !== selectionEnd)
            completionPopup.close()
        onActiveFocusChanged: if (!activeFocus) {
            completionTimer.stop()
            completionPopup.close()
            root.commit()
        }
        Keys.onPressed: function(event) {
            if (completionPopup.opened) {
                if (event.key === Qt.Key_Down || event.key === Qt.Key_Up) {
                    completionPopup.moveSelection(
                        event.key === Qt.Key_Down ? 1 : -1)
                    event.accepted = true
                    return
                }
                if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                    completionPopup.acceptCurrent()
                    event.accepted = true
                    return
                }
                if (event.key === Qt.Key_Escape) {
                    completionTimer.stop()
                    completionPopup.close()
                    event.accepted = true
                    return
                }
            }
            if (event.key === Qt.Key_Slash
                    && (event.modifiers & Qt.ControlModifier)) {
                root.toggleComments()
                event.accepted = true
                return
            }
            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                root.insertIndentedNewline()
                event.accepted = true
                return
            }
            if (event.key === Qt.Key_Backspace && selectionStart === selectionEnd
                    && cursorPosition > 0) {
                const open = text.charAt(cursorPosition - 1)
                const close = open === "(" ? ")" : open === "[" ? "]"
                    : open === "{" ? "}" : open === "\"" ? "\""
                    : open === "'" ? "'" : open === "`" ? "`" : ""
                if (close.length > 0 && text.charAt(cursorPosition) === close) {
                    remove(cursorPosition - 1, cursorPosition + 1)
                    event.accepted = true
                    return
                }
            }
            const typed = event.text
            if (typed === "(" || typed === "[" || typed === "{"
                    || typed === "\"" || typed === "'" || typed === "`") {
                event.accepted = root.insertPair(typed)
                return
            }
            if ((typed === ")" || typed === "]" || typed === "}")
                    && selectionStart === selectionEnd
                    && text.charAt(cursorPosition) === typed) {
                cursorPosition = cursorPosition + 1
                event.accepted = true
            }
        }
        Keys.onTabPressed: function(event) {
            if (!completionPopup.opened)
                root.refreshCompletion(false)
            if (completionPopup.opened) {
                completionPopup.acceptCurrent()
                event.accepted = true
                return
            }
            const spaces = " ".repeat(root.tabWidth)
            insert(cursorPosition, spaces)
            event.accepted = true
        }

        CodeHighlighter {
            document: editor.textDocument
            diagnosticLine: root.diagnosticLine
            diagnosticColumn: root.diagnosticColumn
        }

        Rectangle {
            x: 0
            y: 0
            width: root.showLineNumbers ? 40 : 0
            height: editor.contentHeight + editor.topPadding + editor.bottomPadding
            color: editor.palette.alternateBase
            z: -1
        }
        Repeater {
            model: root.showLineNumbers ? editor.lineCount : 0
            Label {
                required property int index
                x: 4
                y: editor.topPadding + index * implicitHeight
                width: 32
                horizontalAlignment: Text.AlignRight
                text: index + 1
                color: editor.palette.placeholderText
                font: editor.font
            }
        }
    }
}
