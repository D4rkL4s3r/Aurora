use crate::fs_ops::{self, FileEntry, FileKind};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, SystemTime};

/// Filtre rapide sur la taille. Un filtre actif exclut les dossiers.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SizeFilter {
    #[default]
    All,
    Small,
    Medium,
    Large,
}

impl SizeFilter {
    pub(crate) const VARIANTS: [SizeFilter; 4] =
        [Self::All, Self::Small, Self::Medium, Self::Large];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::All => "Taille : toutes",
            Self::Small => "< 1 Mo",
            Self::Medium => "1 à 100 Mo",
            Self::Large => "> 100 Mo",
        }
    }

    fn matches(self, entry: &FileEntry) -> bool {
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
pub(crate) enum DateFilter {
    #[default]
    All,
    Day,
    Week,
    Month,
    Year,
}

impl DateFilter {
    pub(crate) const VARIANTS: [DateFilter; 5] =
        [Self::All, Self::Day, Self::Week, Self::Month, Self::Year];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::All => "Date : toutes",
            Self::Day => "Dernières 24 h",
            Self::Week => "7 derniers jours",
            Self::Month => "30 derniers jours",
            Self::Year => "12 derniers mois",
        }
    }

    fn matches(self, entry: &FileEntry) -> bool {
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
    /// Canal du chargement en cours. Remplacer le receiver abandonne
    /// silencieusement le résultat du chargement précédent.
    pending_load: Option<Receiver<std::io::Result<Vec<FileEntry>>>>,
    pub(crate) load_error: Option<String>,
    /// Texte de la barre d'adresse en cours d'édition (None = breadcrumb affiché).
    pub(crate) address_edit: Option<String>,
    /// Donne le focus au champ d'adresse à la prochaine frame.
    pub(crate) address_focus: bool,
    pub(crate) filter_kind: Option<FileKind>,
    pub(crate) filter_size: SizeFilter,
    pub(crate) filter_date: DateFilter,
}

impl Pane {
    /// Volet « accueil » : aucun dossier chargé, l'écran d'accueil propose
    /// les emplacements de départ.
    pub(crate) fn home() -> Self {
        Self::with_path(PathBuf::new())
    }

    pub(crate) fn is_home(&self) -> bool {
        self.current_path.as_os_str().is_empty()
    }

    pub(crate) fn new(path: PathBuf) -> Self {
        let mut pane = Self::with_path(path);
        pane.reload();
        pane
    }

    fn with_path(path: PathBuf) -> Self {
        Self {
            current_path: path,
            entries: Vec::new(),
            history: Vec::new(),
            selection: HashSet::new(),
            selection_anchor: None,
            search_query: String::new(),
            search_results: None,
            sort_column: SortColumn::Name,
            sort_ascending: true,
            pending_load: None,
            load_error: None,
            address_edit: None,
            address_focus: false,
            filter_kind: None,
            filter_size: SizeFilter::All,
            filter_date: DateFilter::All,
        }
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

    /// Lance le chargement du dossier courant dans un thread de fond.
    /// Le résultat sera intégré par [`Self::poll_load`].
    pub(crate) fn reload(&mut self) {
        if self.is_home() {
            return; // rien à charger sur l'écran d'accueil
        }
        let (tx, rx) = mpsc::channel();
        let path = self.current_path.clone();
        std::thread::spawn(move || {
            let _ = tx.send(fs_ops::list_dir(&path));
        });
        self.pending_load = Some(rx);
    }

    pub(crate) fn is_loading(&self) -> bool {
        self.pending_load.is_some()
    }

    /// Intègre le résultat du chargement en cours s'il est arrivé.
    /// À appeler à chaque frame avant le rendu.
    pub(crate) fn poll_load(&mut self) {
        let Some(rx) = &self.pending_load else {
            return;
        };
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                self.pending_load = None;
                return;
            }
        };
        self.pending_load = None;
        match result {
            Ok(mut entries) => {
                Self::sort_entries(&mut entries, self.sort_column, self.sort_ascending);
                self.entries = entries;
                self.load_error = None;
            }
            Err(e) => {
                self.entries = Vec::new();
                self.load_error = Some(e.to_string());
            }
        }
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
            self.entries.clear();
            self.load_error = None;
            self.selection.clear();
            self.selection_anchor = None;
            self.clear_search();
            self.clear_filters();
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
            self.entries.clear();
            self.load_error = None;
            self.selection.clear();
            self.selection_anchor = None;
            self.clear_search();
            self.clear_filters();
            self.reload();
        }
    }

    pub(crate) fn open_address_bar(&mut self) {
        self.address_edit = Some(self.current_path.display().to_string());
        self.address_focus = true;
    }

    pub(crate) fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_results = None;
    }

    pub(crate) fn has_filters(&self) -> bool {
        self.filter_kind.is_some()
            || self.filter_size != SizeFilter::All
            || self.filter_date != DateFilter::All
    }

    pub(crate) fn clear_filters(&mut self) {
        self.filter_kind = None;
        self.filter_size = SizeFilter::All;
        self.filter_date = DateFilter::All;
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

    /// Vrai si l'entrée passe la recherche instantanée et les filtres rapides.
    fn passes_filters(&self, entry: &FileEntry, query: Option<&str>) -> bool {
        query.is_none_or(|q| entry.name.to_lowercase().contains(q))
            && self.filter_kind.is_none_or(|k| FileKind::of(entry) == k)
            && self.filter_size.matches(entry)
            && self.filter_date.matches(entry)
    }

    /// Indices (dans [`Self::displayed_base`]) des entrées affichées.
    pub(crate) fn displayed_indices(&self) -> Vec<usize> {
        let query = self.instant_filter();
        self.displayed_base()
            .iter()
            .enumerate()
            .filter(|(_, e)| self.passes_filters(e, query.as_deref()))
            .map(|(i, _)| i)
            .collect()
    }

    /// Chemins affichés, dans l'ordre d'affichage.
    pub(crate) fn displayed_paths(&self) -> Vec<PathBuf> {
        let base = self.displayed_base();
        self.displayed_indices()
            .into_iter()
            .map(|i| base[i].path.clone())
            .collect()
    }

    pub(crate) fn displayed_count(&self) -> usize {
        self.displayed_indices().len()
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
        if query.is_empty() || self.is_home() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn wait_for_load(pane: &mut Pane) {
        let start = Instant::now();
        while pane.is_loading() {
            assert!(start.elapsed() < Duration::from_secs(5), "chargement trop long");
            pane.poll_load();
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn pane_loads_directory_asynchronously() {
        let sandbox = crate::fs_ops::temp_sandbox("pane_load");
        std::fs::write(sandbox.join("fichier.txt"), "x").unwrap();

        let mut pane = Pane::new(sandbox.clone());
        wait_for_load(&mut pane);

        assert!(pane.load_error.is_none());
        assert_eq!(pane.entries.len(), 1);
        let _ = std::fs::remove_dir_all(&sandbox);
    }

    #[test]
    fn pane_reports_error_on_missing_directory() {
        let mut pane = Pane::new(PathBuf::from("Z:\\dossier_inexistant_aurora"));
        wait_for_load(&mut pane);

        assert!(pane.load_error.is_some());
        assert!(pane.entries.is_empty());
    }
}
