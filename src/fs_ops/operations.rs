use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
    use crate::fs_ops::temp_sandbox;

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
    fn copy_into_rejects_dir_into_itself() {
        let sandbox = temp_sandbox("copy_self");
        let dir = sandbox.join("dossier");
        fs::create_dir(&dir).unwrap();
        assert!(copy_into(&dir, &dir).is_err());
        let _ = fs::remove_dir_all(&sandbox);
    }
}
