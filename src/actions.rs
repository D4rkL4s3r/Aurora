use crate::pane::SortColumn;
use std::path::PathBuf;

/// Contenu d'un glisser-déposer interne : les chemins de la sélection traînée.
pub(crate) struct DragPayload {
    pub(crate) paths: Vec<PathBuf>,
}

/// Dialogue modal en cours d'affichage.
pub(crate) enum Dialog {
    NewFolder { name: String },
    Rename { target: PathBuf, name: String },
    ConfirmDelete { targets: Vec<PathBuf> },
}

/// Événements émis par le rendu, appliqués à l'état en fin de frame.
pub(crate) enum UiAction {
    /// Rend actif le volet cliqué (vue divisée).
    FocusPane(usize),
    Navigate(PathBuf),
    OpenFile(PathBuf),
    Select { path: PathBuf, ctrl: bool, shift: bool },
    ContextSelect(PathBuf),
    StartRename(PathBuf),
    AskDeleteSelection,
    CopySelection { cut: bool },
    Paste,
    SortBy(SortColumn),
    RunRecursiveSearch,
    /// Dépôt d'un glisser-déposer interne dans le dossier `dest`.
    DropPaths {
        paths: Vec<PathBuf>,
        dest: PathBuf,
        copy: bool,
    },
    OpenTerminal,
    OpenVsCode,
    /// Lance l'application externe configurée à cet index.
    OpenExternalApp(usize),
}
