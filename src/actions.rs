use crate::pane::SortColumn;
use std::path::PathBuf;

/// Dialogue modal en cours d'affichage.
pub(crate) enum Dialog {
    NewFolder { name: String },
    Rename { target: PathBuf, name: String },
    ConfirmDelete { targets: Vec<PathBuf> },
}

/// Événements émis par le rendu, appliqués à l'état en fin de frame.
pub(crate) enum UiAction {
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
    OpenTerminal,
    OpenVsCode,
}
