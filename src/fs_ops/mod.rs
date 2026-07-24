//! Logique fichiers, sans aucune dépendance UI.

mod external;
mod listing;
mod operations;
mod search;

pub use external::{open_in_vscode, open_terminal_here};
pub use listing::{FileEntry, QuickAccessEntry, list_dir, list_drives, list_quick_access};
pub use operations::{
    copy_into, create_dir, delete_to_trash, move_into, open_with_default_app, rename_entry,
};
pub use search::search_recursive;

#[cfg(test)]
pub(crate) fn temp_sandbox(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "aurora_test_{test_name}_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
