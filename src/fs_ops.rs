use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub extension: Option<String>,
}

pub fn list_dir(path: &Path) -> io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let entry_path = entry.path();

        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            extension: entry_path
                .extension()
                .map(|ext| ext.to_string_lossy().into_owned()),
            path: entry_path,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().ok(),
        });
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_dir_current_directory_is_not_empty() {
        let cwd = std::env::current_dir().unwrap();
        let entries = list_dir(&cwd).unwrap();
        assert!(!entries.is_empty());
    }
}
