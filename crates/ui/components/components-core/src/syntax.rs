#[derive(Clone, Copy)]
pub enum SyntaxKind {
    Keyword,
    Boolean,
    Function,
    Variable,
    Number,
    String,
    Comment,
}

#[derive(Clone, Copy)]
pub struct SyntaxSpan {
    pub start_utf16: usize,
    pub length_utf16: usize,
    pub kind: SyntaxKind,
}

pub fn expression_spans(source: &str) -> Vec<SyntaxSpan> {
    let chars = source.char_indices().collect::<Vec<_>>();
    let mut spans = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let (start, character) = chars[index];
        if character == '/' && chars.get(index + 1).is_some_and(|(_, next)| *next == '/') {
            let end = source[start..]
                .find('\n')
                .map_or(source.len(), |offset| start + offset);
            spans.push(span(source, start, end, SyntaxKind::Comment));
            index = chars.partition_point(|(byte, _)| *byte < end);
        } else if character == '/' && chars.get(index + 1).is_some_and(|(_, next)| *next == '*') {
            let end = source[start + 2..]
                .find("*/")
                .map_or(source.len(), |offset| start + 2 + offset + 2);
            spans.push(span(source, start, end, SyntaxKind::Comment));
            index = chars.partition_point(|(byte, _)| *byte < end);
        } else if matches!(character, '"' | '\'' | '`') {
            let quote = character;
            let mut end_index = index + 1;
            let mut escaped = false;
            while end_index < chars.len() {
                let (_, next) = chars[end_index];
                end_index += 1;
                if next == quote && !escaped {
                    break;
                }
                escaped = next == '\\' && !escaped;
                if next != '\\' {
                    escaped = false;
                }
            }
            let end = chars.get(end_index).map_or(source.len(), |(byte, _)| *byte);
            spans.push(span(source, start, end, SyntaxKind::String));
            index = end_index;
        } else if character.is_ascii_digit()
            && index
                .checked_sub(1)
                .is_none_or(|previous| !is_identifier(chars[previous].1))
        {
            let mut end_index = index + 1;
            while chars
                .get(end_index)
                .is_some_and(|(_, next)| next.is_ascii_digit() || *next == '.')
            {
                end_index += 1;
            }
            if chars
                .get(end_index)
                .is_some_and(|(_, next)| matches!(*next, 'e' | 'E'))
            {
                let exponent = end_index;
                end_index += 1;
                if chars
                    .get(end_index)
                    .is_some_and(|(_, next)| matches!(*next, '+' | '-'))
                {
                    end_index += 1;
                }
                let digits = end_index;
                while chars
                    .get(end_index)
                    .is_some_and(|(_, next)| next.is_ascii_digit())
                {
                    end_index += 1;
                }
                if digits == end_index {
                    end_index = exponent;
                }
            }
            let end = chars.get(end_index).map_or(source.len(), |(byte, _)| *byte);
            if chars
                .get(end_index)
                .is_none_or(|(_, next)| !is_identifier(*next))
            {
                spans.push(span(source, start, end, SyntaxKind::Number));
            }
            index = end_index;
        } else if character == '_' || character.is_alphabetic() {
            let mut end_index = index + 1;
            while chars
                .get(end_index)
                .is_some_and(|(_, next)| *next == '_' || next.is_alphanumeric())
            {
                end_index += 1;
            }
            let end = chars.get(end_index).map_or(source.len(), |(byte, _)| *byte);
            let word = &source[start..end];
            let kind = if matches!(word, "true" | "false") {
                Some(SyntaxKind::Boolean)
            } else if matches!(
                word,
                "break"
                    | "const"
                    | "continue"
                    | "else"
                    | "export"
                    | "fn"
                    | "for"
                    | "if"
                    | "in"
                    | "let"
                    | "loop"
                    | "return"
                    | "while"
            ) {
                Some(SyntaxKind::Keyword)
            } else if matches!(
                word,
                "Fraction"
                    | "abs"
                    | "clamp"
                    | "cos"
                    | "int"
                    | "lerp"
                    | "gray"
                    | "graya"
                    | "hsv"
                    | "hsva"
                    | "oklab"
                    | "oklaba"
                    | "pow"
                    | "random"
                    | "rgb"
                    | "rgba"
                    | "shake"
                    | "sin"
                    | "sqrt"
                    | "tan"
                    | "vol"
            ) {
                Some(SyntaxKind::Function)
            } else if matches!(
                word,
                "canvas_height"
                    | "canvas_width"
                    | "duration"
                    | "fps"
                    | "local_t"
                    | "media_height"
                    | "media_width"
                    | "seed"
                    | "source_height"
                    | "source_width"
                    | "t"
                    | "time"
                    | "value"
                    | "a"
                    | "b"
                    | "g"
                    | "r"
                    | "x"
                    | "y"
                    | "z"
            ) {
                Some(SyntaxKind::Variable)
            } else {
                None
            };
            if let Some(kind) = kind {
                spans.push(span(source, start, end, kind));
            }
            index = end_index;
        } else {
            index += 1;
        }
    }
    spans
}

fn is_identifier(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}

fn span(source: &str, start: usize, end: usize, kind: SyntaxKind) -> SyntaxSpan {
    SyntaxSpan {
        start_utf16: source[..start].encode_utf16().count(),
        length_utf16: source[start..end].encode_utf16().count(),
        kind,
    }
}
