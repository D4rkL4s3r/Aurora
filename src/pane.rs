use crate::fs_ops::{self, FileEntry};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortColumn {
    Name,
    Size,
    Type,
    Modified,
}

/// État d'une vue de navigation : chemin courant, contenu, sélection,
/// historique, recherche et tri. `App` en possède une instance aujourd'hui,
/// plusieurs demain (vue divisée, onglets).
pub(crate) struct Pane {
    pub(crate) current_path: PathBuf,
    pub(crate) entries: Vec<FileEntry>,
    pub(crate) history: Vec<PathBuf>,
    pub(crate) selection: HashSet<PathBuf>,
    pub(crate) selection_anchor: Option<PathBuf>,
    pub(crate) search_query: String,
    pub(crate) search_results: Option<Vec<FileEntry>>,
    pub(crate) sort_column: SortColumn,
    pub(crate) sort_ascending: bool,
}

impl Pane {
    pub(crate) fn new(path: PathBuf) -> Self {
        let mut pane = Self {
            current_path: path,
            entries: Vec::new(),
            history: Vec::new(),
            selection: HashSet::new(),
            selection_anchor: None,
            search_query: String::new(),
            search_results: None,
            sort_column: SortColumn::Name,
            sort_ascending: true,
        };
        pane.reload();
        pane
    }

    fn sort_entries(entries: &mut [FileEntry], column: SortColumn, ascending: bool) {
        entries.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir).then_with(|| {
                let ord = match column {
                    SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortColumn::Size => a.size.cmp(&b.size),
                    SortColumn::Type => {
                        let ext_a = a.extension.as_deref().unwrap_or("").to_lowercase();
                        let ext_b = b.extension.as_deref().unwrap_or("").to_lowercase();
                        ext_a
                            .cmp(&ext_b)
                            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                    }
                    SortColumn::Modified => a.modified.cmp(&b.modified),
                };
                if ascending { ord } else { ord.reverse() }
            })
        });
    }

    pub(crate) fn reload(&mut self) {
        self.entries = fs_ops::list_dir(&self.current_path).unwrap_or_default();
        Self::sort_entries(&mut self.entries, self.sort_column, self.sort_ascending);
        if let Some(results) = &mut self.search_results {
            results.retain(|e| e.path.exists());
        }
        let mut existing: HashSet<&PathBuf> = self.entries.iter().map(|e| &e.path).collect();
        if let Some(results) = &self.search_results {
            existing.extend(results.iter().map(|e| &e.path));
        }
        self.selection.retain(|p| existing.contains(p));
    }

    pub(crate) fn navigate_to(&mut self, path: PathBuf) {
        if path != self.current_path {
            self.history.push(self.current_path.clone());
            self.current_path = path;
            self.selection.clear();
            self.selection_anchor = None;
            self.clear_search();
            self.reload();
        }
    }

    pub(crate) fn go_parent(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.navigate_to(parent.to_path_buf());
        }
    }

    pub(crate) fn go_back(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.current_path = previous;
            self.selection.clear();
            self.selection_anchor = None;
            self.clear_search();
            self.reload();
        }
    }

    pub(crate) fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_results = None;
    }

    /// Liste actuellement affichée : résultats de recherche récursive,
    /// ou contenu du dossier (filtré par la recherche instantanée).
    pub(crate) fn displayed_base(&self) -> &[FileEntry] {
        match &self.search_results {
            Some(results) => results,
            None => &self.entries,
        }
    }

    pub(crate) fn instant_filter(&self) -> Option<String> {
        let query = self.search_query.trim();
        (self.search_results.is_none() && !query.is_empty()).then(|| query.to_lowercase())
    }

    /// Chemins affichés, dans l'ordre d'affichage.
    pub(crate) fn displayed_paths(&self) -> Vec<PathBuf> {
        let base = self.displayed_base();
        match self.instant_filter() {
            Some(query) => base
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&query))
                .map(|e| e.path.clone())
                .collect(),
            None => base.iter().map(|e| e.path.clone()).collect(),
        }
    }

    pub(crate) fn displayed_count(&self) -> usize {
        match self.instant_filter() {
            Some(query) => self
                .displayed_base()
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&query))
                .count(),
            None => self.displayed_base().len(),
        }
    }

    /// Chemins sélectionnés, dans l'ordre d'affichage.
    pub(crate) fn selected_in_order(&self) -> Vec<PathBuf> {
        self.displayed_paths()
            .into_iter()
            .filter(|p| self.selection.contains(p))
            .collect()
    }

    pub(crate) fn select(&mut self, path: PathBuf, ctrl: bool, shift: bool) {
        let displayed = self.displayed_paths();
        if shift {
            let anchor = self
                .selection_anchor
                .as_ref()
                .and_then(|a| displayed.iter().position(|p| p == a));
            let clicked = displayed.iter().position(|p| p == &path);
            if let (Some(anchor), Some(clicked)) = (anchor, clicked) {
                let (from, to) = (anchor.min(clicked), anchor.max(clicked));
                if !ctrl {
                    self.selection.clear();
                }
                for p in &displayed[from..=to] {
                    self.selection.insert(p.clone());
                }
                return; // l'ancre ne bouge pas : Shift+clic successifs étendent depuis la même origine
            }
        }
        if ctrl {
            if !self.selection.remove(&path) {
                self.selection.insert(path.clone());
            }
        } else {
            self.selection.clear();
            self.selection.insert(path.clone());
        }
        self.selection_anchor = Some(path);
    }

    pub(crate) fn select_all(&mut self) {
        self.selection = self.displayed_paths().into_iter().collect();
    }

    pub(crate) fn set_selection_to(&mut self, path: PathBuf) {
        self.selection.clear();
        self.selection.insert(path.clone());
        self.selection_anchor = Some(path);
    }

    pub(crate) fn set_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = true;
        }
        Self::sort_entries(&mut self.entries, self.sort_column, self.sort_ascending);
        if let Some(results) = &mut self.search_results {
            Self::sort_entries(results, self.sort_column, self.sort_ascending);
        }
    }

    /// Lance la recherche récursive et retourne le message de statut.
    pub(crate) fn run_recursive_search(&mut self) -> Option<String> {
        const MAX_DEPTH: usize = 8;
        const MAX_RESULTS: usize = 300;
        let query = self.search_query.trim().to_owned();
        if query.is_empty() {
            return None;
        }
        let mut results =
            fs_ops::search_recursive(&self.current_path, &query, MAX_DEPTH, MAX_RESULTS);
        Self::sort_entries(&mut results, self.sort_column, self.sort_ascending);
        let capped = if results.len() >= MAX_RESULTS {
            " (limité à 300)"
        } else {
            ""
        };
        let status = format!(
            "{} résultat(s) dans les sous-dossiers{capped}",
            results.len()
        );
        self.search_results = Some(results);
        self.selection.clear();
        self.selection_anchor = None;
        Some(status)
    }
}
