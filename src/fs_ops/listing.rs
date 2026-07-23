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

/// Liste les lecteurs Windows disponibles (ex. `C:\`, `D:\`), triés par lettre.
pub fn list_drives() -> Vec<PathBuf> {
    let mask = unsafe { windows::Win32::Storage::FileSystem::GetLogicalDrives() };

    (0..26)
        .filter(|i| mask & (1 << i) != 0)
        .map(|i| PathBuf::from(format!("{}:\\", (b'A' + i as u8) as char)))
        .collect()
}

pub struct QuickAccessEntry {
    pub name: &'static str,
    pub path: PathBuf,
}

/// Liste les dossiers "Accès rapide" (Bureau, Documents, Téléchargements...)
/// via les Known Folders Windows, pour suivre une éventuelle redirection
/// (OneDrive, profil personnalisé, etc.).
pub fn list_quick_access() -> Vec<QuickAccessEntry> {
    use windows::Win32::UI::Shell::{
        FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_Downloads, FOLDERID_Music,
        FOLDERID_Pictures, FOLDERID_Videos,
    };

    [
        ("Bureau", &FOLDERID_Desktop),
        ("Documents", &FOLDERID_Documents),
        ("Téléchargements", &FOLDERID_Downloads),
        ("Images", &FOLDERID_Pictures),
        ("Musique", &FOLDERID_Music),
        ("Vidéos", &FOLDERID_Videos),
    ]
    .into_iter()
    .filter_map(|(name, id)| known_folder_path(id).map(|path| QuickAccessEntry { name, path }))
    .collect()
}

fn known_folder_path(id: &windows::core::GUID) -> Option<PathBuf> {
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{KF_FLAG_DEFAULT, SHGetKnownFolderPath};

    unsafe {
        let pwstr = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let path = pwstr.to_string().ok().map(PathBuf::from);
        CoTaskMemFree(Some(pwstr.0 as *const _));
        path
    }
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
