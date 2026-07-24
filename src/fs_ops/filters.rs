//! Filtres rapides sur les entrées (taille, date) — logique pure, sans UI.
//! Le filtre par catégorie est porté par [`super::FileKind`].

use super::FileEntry;
use std::time::{Duration, SystemTime};

/// Filtre rapide sur la taille. Un filtre actif exclut les dossiers.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SizeFilter {
    #[default]
    All,
    Small,
    Medium,
    Large,
}

impl SizeFilter {
    pub const VARIANTS: [SizeFilter; 4] = [Self::All, Self::Small, Self::Medium, Self::Large];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "Taille : toutes",
            Self::Small => "< 1 Mo",
            Self::Medium => "1 à 100 Mo",
            Self::Large => "> 100 Mo",
        }
    }

    pub fn matches(self, entry: &FileEntry) -> bool {
        const MB: u64 = 1024 * 1024;
        if entry.is_dir {
            return self == Self::All;
        }
        match self {
            Self::All => true,
            Self::Small => entry.size < MB,
            Self::Medium => (MB..100 * MB).contains(&entry.size),
            Self::Large => entry.size >= 100 * MB,
        }
    }
}

/// Filtre rapide sur la date de modification.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DateFilter {
    #[default]
    All,
    Day,
    Week,
    Month,
    Year,
}

impl DateFilter {
    pub const VARIANTS: [DateFilter; 5] =
        [Self::All, Self::Day, Self::Week, Self::Month, Self::Year];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "Date : toutes",
            Self::Day => "Dernières 24 h",
            Self::Week => "7 derniers jours",
            Self::Month => "30 derniers jours",
            Self::Year => "12 derniers mois",
        }
    }

    pub fn matches(self, entry: &FileEntry) -> bool {
        let days = match self {
            Self::All => return true,
            Self::Day => 1,
            Self::Week => 7,
            Self::Month => 30,
            Self::Year => 365,
        };
        let Some(modified) = entry.modified else {
            return false;
        };
        SystemTime::now()
            .duration_since(modified)
            .map(|age| age <= Duration::from_secs(days * 24 * 3600))
            .unwrap_or(true) // date dans le futur : on ne cache pas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn entry(is_dir: bool, size: u64, age_secs: Option<u64>) -> FileEntry {
        FileEntry {
            name: "test".to_owned(),
            path: PathBuf::from("test"),
            is_dir,
            size,
            modified: age_secs.map(|s| SystemTime::now() - Duration::from_secs(s)),
            extension: None,
        }
    }

    #[test]
    fn size_filter_boundaries() {
        const MB: u64 = 1024 * 1024;
        assert!(SizeFilter::Small.matches(&entry(false, MB - 1, None)));
        assert!(!SizeFilter::Small.matches(&entry(false, MB, None)));
        assert!(SizeFilter::Medium.matches(&entry(false, MB, None)));
        assert!(SizeFilter::Large.matches(&entry(false, 100 * MB, None)));
        // Un filtre de taille actif exclut les dossiers.
        assert!(!SizeFilter::Small.matches(&entry(true, 0, None)));
        assert!(SizeFilter::All.matches(&entry(true, 0, None)));
    }

    #[test]
    fn date_filter_ages() {
        assert!(DateFilter::Day.matches(&entry(false, 0, Some(3600))));
        assert!(!DateFilter::Day.matches(&entry(false, 0, Some(2 * 24 * 3600))));
        assert!(DateFilter::Week.matches(&entry(false, 0, Some(2 * 24 * 3600))));
        // Sans date de modification, seul « toutes » laisse passer.
        assert!(DateFilter::All.matches(&entry(false, 0, None)));
        assert!(!DateFilter::Month.matches(&entry(false, 0, None)));
    }
}
