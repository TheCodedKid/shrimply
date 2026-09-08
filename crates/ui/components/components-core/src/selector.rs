pub const MAX_OPTIONS_WITHOUT_SEARCH: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringChoice {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchChoices {
    pub choices: Vec<StringChoice>,
    keywords: Vec<String>,
}

impl From<Vec<StringChoice>> for SearchChoices {
    fn from(choices: Vec<StringChoice>) -> Self {
        Self {
            choices,
            keywords: Vec::new(),
        }
    }
}

impl SearchChoices {
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        assert!(
            keywords.is_empty() || keywords.len() == self.choices.len(),
            "search keywords must correspond to choices"
        );
        self.keywords = keywords;
        self
    }

    pub fn matching_indices(&self, query: &str) -> Vec<usize> {
        ranked_indices(
            self.choices.iter().map(|choice| choice.label.as_str()),
            &self.keywords,
            query,
        )
    }
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
    label.to_lowercase().contains(&query.trim().to_lowercase())
}

pub fn search_rank<'a>(
    label: &str,
    keywords: impl IntoIterator<Item = &'a str>,
    query: &str,
) -> Option<u8> {
    if matches_query(label, query) {
        Some(0)
    } else if keywords
        .into_iter()
        .any(|keyword| matches_query(keyword, query))
    {
        Some(1)
    } else {
        None
    }
}

pub fn ranked_matching_indices(
    labels: &[String],
    keyword_groups: &[String],
    query: &str,
) -> Vec<usize> {
    ranked_indices(labels.iter().map(String::as_str), keyword_groups, query)
}

fn ranked_indices<'a>(
    labels: impl Iterator<Item = &'a str>,
    keyword_groups: &[String],
    query: &str,
) -> Vec<usize> {
    let mut matches = labels
        .enumerate()
        .filter_map(|(index, label)| {
            let keywords = keyword_groups.get(index).map_or("", String::as_str);
            search_rank(label, [keywords], query).map(|rank| (rank, index))
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|(rank, _)| *rank);
    matches.into_iter().map(|(_, index)| index).collect()
}

pub fn adjacent_matching_index(
    labels: &[String],
    query: &str,
    current: Option<usize>,
    forward: bool,
) -> Option<usize> {
    let matches = labels
        .iter()
        .enumerate()
        .filter(|(_, label)| matches_query(label, query))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let boundary = || {
        if forward {
            matches.first().copied()
        } else {
            matches.last().copied()
        }
    };
    let Some(current) = current else {
        return boundary();
    };
    if forward {
        matches
            .iter()
            .copied()
            .find(|index| *index > current)
            .or_else(|| matches.contains(&current).then_some(current))
            .or_else(boundary)
    } else {
        matches
            .iter()
            .rev()
            .copied()
            .find(|index| *index < current)
            .or_else(|| matches.contains(&current).then_some(current))
            .or_else(boundary)
    }
}
