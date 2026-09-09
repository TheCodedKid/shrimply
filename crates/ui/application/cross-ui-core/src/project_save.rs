use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectFormat {
    Shrimply,
    Json,
}

impl ProjectFormat {
    pub const ALL: [Self; 2] = [Self::Shrimply, Self::Json];

    pub fn from_path(path: &Path) -> Self {
        if path.extension().is_some_and(|extension| {
            extension.eq_ignore_ascii_case("sjson") || extension.eq_ignore_ascii_case("json")
        }) {
            Self::Json
        } else {
            Self::Shrimply
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Shrimply => "shrimp",
            Self::Json => "sjson",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Shrimply => "Shrimply (.shrimp)",
            Self::Json => "Shrimply JSON (.sjson)",
        }
    }

    pub fn normalize_path(self, mut path: PathBuf) -> PathBuf {
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case(self.extension()))
        {
            path.set_extension(self.extension());
        }
        path
    }
}
