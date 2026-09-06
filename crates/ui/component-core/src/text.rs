#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypoMark {
    pub start: i32,
    pub end: i32,
    pub message: String,
    pub corrections: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct TextCommit {
    dirty: bool,
    latest: String,
    committed: String,
}

impl TextCommit {
    pub fn new(value: &str) -> Self {
        Self {
            dirty: false,
            latest: value.to_string(),
            committed: value.to_string(),
        }
    }

    pub fn changed(&mut self, value: String) {
        self.latest = value;
        self.dirty = true;
    }

    pub fn synchronize(&mut self, value: String) {
        self.dirty = false;
        self.latest.clone_from(&value);
        self.committed = value;
    }

    pub fn take_commit(&mut self) -> bool {
        if !self.dirty || self.latest == self.committed {
            self.dirty = false;
            return false;
        }
        self.committed.clone_from(&self.latest);
        self.dirty = false;
        true
    }
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
