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

/// Recherche récursive par nom sous `root`, insensible à la casse.
/// La profondeur et le nombre de résultats sont plafonnés pour rester réactif.
pub fn search_recursive(
    root: &Path,
    query: &str,
    max_depth: usize,
    max_results: usize,
) -> Vec<FileEntry> {
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for entry in walkdir::WalkDir::new(root)
        .min_depth(1)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if results.len() >= max_results {
            break;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.to_lowercase().contains(&query) {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let path = entry.into_path();
        results.push(FileEntry {
            name,
            extension: path
                .extension()
                .map(|ext| ext.to_string_lossy().into_owned()),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().ok(),
            path,
        });
    }

    results
}

/// Ouvre un fichier avec l'application par défaut du système.
pub fn open_with_default_app(path: &Path) -> io::Result<()> {
    open::that(path)
}

/// Crée un dossier `name` dans `parent` et retourne son chemin.
pub fn create_dir(parent: &Path, name: &str) -> io::Result<PathBuf> {
    let path = parent.join(name);
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Un élément porte déjà ce nom",
        ));
    }
    fs::create_dir(&path)?;
    Ok(path)
}

/// Renomme `path` en `new_name` (dans le même dossier parent).
pub fn rename_entry(path: &Path, new_name: &str) -> io::Result<PathBuf> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Impossible de renommer la racine")
    })?;
    let new_path = parent.join(new_name);
    if new_path == path {
        return Ok(new_path);
    }
    if new_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Un élément porte déjà ce nom",
        ));
    }
    fs::rename(path, &new_path)?;
    Ok(new_path)
}

/// Envoie les chemins donnés à la corbeille Windows.
pub fn delete_to_trash(paths: &[PathBuf]) -> io::Result<()> {
    trash::delete_all(paths).map_err(io::Error::other)
}

/// Copie `src` (fichier ou dossier, récursivement) dans le dossier `dest_dir`.
/// En cas de conflit de nom, un suffixe « - copie » est ajouté.
pub fn copy_into(src: &Path, dest_dir: &Path) -> io::Result<PathBuf> {
    let name = file_name_of(src)?;
    if src.is_dir() && dest_dir.starts_with(src) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Impossible de copier un dossier dans lui-même",
        ));
    }
    let dest = unique_destination(dest_dir, name);
    if src.is_dir() {
        copy_dir_recursive(src, &dest)?;
    } else {
        fs::copy(src, &dest)?;
    }
    Ok(dest)
}

/// Déplace `src` dans le dossier `dest_dir`. Tente un simple rename ;
/// si ça échoue (autre volume), copie puis supprime la source.
pub fn move_into(src: &Path, dest_dir: &Path) -> io::Result<PathBuf> {
    let name = file_name_of(src)?;
    if src.parent() == Some(dest_dir) {
        return Ok(src.to_path_buf());
    }
    if src.is_dir() && dest_dir.starts_with(src) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Impossible de déplacer un dossier dans lui-même",
        ));
    }
    let dest = unique_destination(dest_dir, name);
    match fs::rename(src, &dest) {
        Ok(()) => Ok(dest),
        Err(_) => {
            if src.is_dir() {
                copy_dir_recursive(src, &dest)?;
                fs::remove_dir_all(src)?;
            } else {
                fs::copy(src, &dest)?;
                fs::remove_file(src)?;
            }
            Ok(dest)
        }
    }
}

fn file_name_of(path: &Path) -> io::Result<&std::ffi::OsStr> {
    path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Chemin sans nom de fichier")
    })
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Trouve un nom libre dans `dest_dir` pour `file_name`
/// (« nom - copie.ext », puis « nom - copie (2).ext », etc.).
fn unique_destination(dest_dir: &Path, file_name: &std::ffi::OsStr) -> PathBuf {
    let candidate = dest_dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }
    let file_name = Path::new(file_name);
    let stem = file_name
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = file_name
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    for n in 1u32.. {
        let name = if n == 1 {
            format!("{stem} - copie{ext}")
        } else {
            format!("{stem} - copie ({n}){ext}")
        };
        let candidate = dest_dir.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!()
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

    fn temp_sandbox(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aurora_test_{test_name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn create_dir_then_rename() {
        let sandbox = temp_sandbox("create_rename");
        let created = create_dir(&sandbox, "alpha").unwrap();
        assert!(created.is_dir());
        assert!(create_dir(&sandbox, "alpha").is_err());

        let renamed = rename_entry(&created, "beta").unwrap();
        assert!(renamed.is_dir());
        assert!(!created.exists());
        let _ = fs::remove_dir_all(&sandbox);
    }

    #[test]
    fn copy_into_adds_copie_suffix_on_conflict() {
        let sandbox = temp_sandbox("copy_conflict");
        let src = sandbox.join("note.txt");
        fs::write(&src, "contenu").unwrap();

        let copy1 = copy_into(&src, &sandbox).unwrap();
        assert_eq!(copy1.file_name().unwrap(), "note - copie.txt");
        let copy2 = copy_into(&src, &sandbox).unwrap();
        assert_eq!(copy2.file_name().unwrap(), "note - copie (2).txt");
        assert_eq!(fs::read_to_string(&copy2).unwrap(), "contenu");
        let _ = fs::remove_dir_all(&sandbox);
    }

    #[test]
    fn move_into_moves_directory_with_contents() {
        let sandbox = temp_sandbox("move_dir");
        let src_dir = sandbox.join("source");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("fichier.txt"), "x").unwrap();
        let dest_dir = sandbox.join("destination");
        fs::create_dir(&dest_dir).unwrap();

        let moved = move_into(&src_dir, &dest_dir).unwrap();
        assert!(!src_dir.exists());
        assert!(moved.join("fichier.txt").is_file());
        let _ = fs::remove_dir_all(&sandbox);
    }

    #[test]
    fn search_recursive_finds_nested_file_case_insensitive() {
        let sandbox = temp_sandbox("search");
        let nested = sandbox.join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("Rapport-Final.txt"), "x").unwrap();
        fs::write(sandbox.join("autre.txt"), "x").unwrap();

        let results = search_recursive(&sandbox, "rapport", 8, 300);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Rapport-Final.txt");

        let capped = search_recursive(&sandbox, "txt", 8, 1);
        assert_eq!(capped.len(), 1);
        let _ = fs::remove_dir_all(&sandbox);
    }

    #[test]
    fn copy_into_rejects_dir_into_itself() {
        let sandbox = temp_sandbox("copy_self");
        let dir = sandbox.join("dossier");
        fs::create_dir(&dir).unwrap();
        assert!(copy_into(&dir, &dir).is_err());
        let _ = fs::remove_dir_all(&sandbox);
    }
}
