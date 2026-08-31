#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypoMark {
    pub start: i32,
    pub end: i32,
    pub message: String,
    pub corrections: Vec<String>,
}

pub fn limited_text(text: &str, max_length: Option<usize>) -> String {
    match max_length {
        Some(max_length) => text.chars().take(max_length).collect(),
        None => text.to_string(),
    }
}

pub fn typo_marks(text: &str) -> Vec<TypoMark> {
    let tokenizer = typos::tokens::Tokenizer::new();
    let mut marks = Vec::new();
    for ident in tokenizer.parse_str(text) {
        for word in ident.split() {
            let Some(corrections) = typos_dict::WORD.find(&unicase::UniCase::new(word.token()))
            else {
                continue;
            };
            let start = char_offset(text, word.offset());
            let end = char_offset(text, word.offset() + word.token().len());
            marks.push(TypoMark {
                start,
                end,
                message: if corrections.is_empty() {
                    format!("Possible typo: {}", word.token())
                } else {
                    format!(
                        "Possible typo: {} -> {}",
                        word.token(),
                        corrections.join(", ")
                    )
                },
                corrections: corrections.iter().map(ToString::to_string).collect(),
            });
        }
    }
    marks
}

fn char_offset(text: &str, byte_offset: usize) -> i32 {
    text[..byte_offset].chars().count().min(i32::MAX as usize) as i32
}
