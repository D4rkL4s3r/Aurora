use crate::pane::Pane;
use std::path::PathBuf;

/// Un onglet : un ou deux volets (vue divisée) et l'index du volet actif.
pub(crate) struct Tab {
    pub(crate) panes: Vec<Pane>,
    pub(crate) active_pane: usize,
}

impl Tab {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            panes: vec![Pane::new(path)],
            active_pane: 0,
        }
    }

    pub(crate) fn pane(&self) -> &Pane {
        &self.panes[self.active_pane]
    }

    pub(crate) fn pane_mut(&mut self) -> &mut Pane {
        &mut self.panes[self.active_pane]
    }

    pub(crate) fn is_split(&self) -> bool {
        self.panes.len() > 1
    }

    pub(crate) fn toggle_split(&mut self) {
        if self.is_split() {
            self.panes.truncate(1);
            self.active_pane = 0;
        } else {
            let path = self.panes[0].current_path.clone();
            self.panes.push(Pane::new(path));
            self.active_pane = 1;
        }
    }

    /// Titre affiché dans la barre d'onglets : dossier courant du volet actif.
    pub(crate) fn title(&self) -> String {
        let path = &self.pane().current_path;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        if name.chars().count() <= 18 {
            name
        } else {
            let mut short: String = name.chars().take(17).collect();
            short.push('…');
            short
        }
    }
}
