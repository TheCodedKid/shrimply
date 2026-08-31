pub const MAX_OPTIONS_WITHOUT_SEARCH: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringChoice {
    pub value: String,
    pub label: String,
}

pub fn identity_choices(options: Vec<String>) -> Vec<StringChoice> {
    options
        .into_iter()
        .map(|option| StringChoice {
            value: option.clone(),
            label: option,
        })
        .collect()
}

pub fn selected_index(value: &str, choices: &[StringChoice]) -> usize {
    matching_index(value, choices).unwrap_or_default()
}

pub fn matching_index(value: &str, choices: &[StringChoice]) -> Option<usize> {
    choices.iter().position(|choice| choice.value == value)
}

pub fn searchable(choice_count: usize) -> bool {
    choice_count > MAX_OPTIONS_WITHOUT_SEARCH
}

pub fn matches_query(label: &str, query: &str) -> bool {
    label.to_lowercase().contains(&query.to_lowercase())
}
