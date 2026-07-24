//! Lancement d'applications externes (terminal, éditeur) dans un dossier donné.

use std::io;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Le processus lancé reçoit sa propre console (sinon un shell lancé depuis
/// une app GUI n'a aucune fenêtre).
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
/// Pas de console du tout (pour les .cmd relayés par cmd.exe).
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Cherche le premier des exécutables donnés dans les dossiers du PATH.
fn find_in_path(names: &[&str]) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .find_map(|dir| names.iter().map(|n| dir.join(n)).find(|c| c.is_file()))
}

/// Ouvre un terminal dans `dir` : Windows Terminal si disponible, sinon
/// PowerShell, sinon cmd.
///
/// `wt.exe` est tenté directement plutôt que cherché dans le PATH : son alias
/// WindowsApps est un point de reparse que `is_file()` ne sait pas lire.
pub fn open_terminal_here(dir: &Path) -> io::Result<()> {
    if Command::new("wt.exe").arg("-d").arg(dir).spawn().is_ok() {
        return Ok(());
    }
    for shell in ["powershell.exe", "cmd.exe"] {
        if Command::new(shell)
            .current_dir(dir)
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
    }
    Err(io::Error::other("Aucun terminal trouvé"))
}

/// Localise VS Code : installation utilisateur habituelle, sinon PATH.
fn vscode_path() -> Option<PathBuf> {
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let user_install = PathBuf::from(local).join("Programs/Microsoft VS Code/Code.exe");
        if user_install.is_file() {
            return Some(user_install);
        }
    }
    find_in_path(&["Code.exe", "code.cmd"])
}

/// Ouvre `dir` dans VS Code.
pub fn open_in_vscode(dir: &Path) -> io::Result<()> {
    let code = vscode_path().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "VS Code introuvable (ni installé, ni dans le PATH)",
        )
    })?;
    Command::new(code)
        .arg(dir)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
}

/// Application externe configurée par l'utilisateur (9.3).
#[derive(Clone, Default)]
pub struct ExternalApp {
    pub name: String,
    pub command: String,
}

fn external_apps_path() -> Option<PathBuf> {
    crate::fs_ops::config_dir().map(|dir| dir.join("external_apps.conf"))
}

/// Charge la liste (`nom=chemin_exe`, une ligne par application).
pub fn load_external_apps() -> Vec<ExternalApp> {
    let Some(path) = external_apps_path() else {
        return Vec::new();
    };
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|line| {
            let (name, command) = line.split_once('=')?;
            let (name, command) = (name.trim(), command.trim());
            (!name.is_empty() && !command.is_empty()).then(|| ExternalApp {
                name: name.to_owned(),
                command: command.to_owned(),
            })
        })
        .collect()
}

pub fn save_external_apps(apps: &[ExternalApp]) {
    let Some(path) = external_apps_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content: String = apps
        .iter()
        .map(|a| format!("{}={}\n", a.name.trim(), a.command.trim()))
        .collect();
    let _ = std::fs::write(path, content);
}

/// Lance une application configurée avec le dossier courant en argument
/// (et comme répertoire de travail).
pub fn launch_external(app: &ExternalApp, dir: &Path) -> io::Result<()> {
    Command::new(&app.command)
        .arg(dir)
        .current_dir(dir)
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_in_path_locates_cmd() {
        // cmd.exe est toujours dans System32, lui-même toujours dans le PATH.
        assert!(find_in_path(&["cmd.exe"]).is_some());
        assert!(find_in_path(&["executable_inexistant_aurora.exe"]).is_none());
    }
}
