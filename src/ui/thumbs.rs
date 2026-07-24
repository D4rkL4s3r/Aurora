//! Miniatures d'images pour la vue grille, chargées dans un thread de fond
//! pour ne jamais bloquer l'interface. Cache par chemin, échecs mémorisés.

use crate::fs_ops::FileEntry;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

/// Côté maximal d'une miniature (px). Assez pour la carte de la grille.
const THUMB_SIZE: u32 = 96;
/// Au-delà, le cache est vidé d'un bloc (dossiers de milliers d'images).
const MAX_CACHED: usize = 512;

enum ThumbState {
    Pending,
    Ready(egui::TextureHandle),
    Failed,
}

pub(crate) struct ThumbnailCache {
    states: HashMap<PathBuf, ThumbState>,
    request_tx: Sender<PathBuf>,
    result_rx: Receiver<(PathBuf, Option<egui::ColorImage>)>,
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        let (request_tx, request_rx) = channel::<PathBuf>();
        let (result_tx, result_rx) = channel();
        std::thread::spawn(move || {
            for path in request_rx.iter() {
                let image = load_thumbnail(&path);
                if result_tx.send((path, image)).is_err() {
                    break; // l'application est fermée
                }
            }
        });
        Self {
            states: HashMap::new(),
            request_tx,
            result_rx,
        }
    }
}

impl ThumbnailCache {
    /// Extensions pour lesquelles une miniature est tentée (celles que la
    /// crate `image` sait décoder parmi nos catégories d'images).
    pub(crate) fn is_supported_image(entry: &FileEntry) -> bool {
        !entry.is_dir
            && matches!(
                entry
                    .extension
                    .as_deref()
                    .map(|e| e.to_lowercase())
                    .as_deref(),
                Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "tif" | "tiff")
            )
    }

    /// Intègre les miniatures terminées. À appeler chaque frame avant le rendu.
    pub(crate) fn poll(&mut self, ctx: &egui::Context) {
        while let Ok((path, image)) = self.result_rx.try_recv() {
            let state = match image {
                Some(image) => ThumbState::Ready(ctx.load_texture(
                    format!("thumb:{}", path.display()),
                    image,
                    egui::TextureOptions::LINEAR,
                )),
                None => ThumbState::Failed,
            };
            self.states.insert(path, state);
        }
        if self
            .states
            .values()
            .any(|s| matches!(s, ThumbState::Pending))
        {
            // Des chargements sont en cours : repasser bientôt les intégrer.
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    /// Miniature prête pour cette entrée, en demandant son chargement au
    /// premier appel. À n'appeler que pour des cartes visibles.
    pub(crate) fn get(&mut self, entry: &FileEntry) -> Option<egui::TextureHandle> {
        if !Self::is_supported_image(entry) {
            return None;
        }
        if !self.states.contains_key(&entry.path) {
            if self.states.len() >= MAX_CACHED {
                self.states.clear();
            }
            let _ = self.request_tx.send(entry.path.clone());
            self.states.insert(entry.path.clone(), ThumbState::Pending);
            return None;
        }
        match self.states.get(&entry.path) {
            Some(ThumbState::Ready(texture)) => Some(texture.clone()),
            _ => None,
        }
    }
}

fn load_thumbnail(path: &std::path::Path) -> Option<egui::ColorImage> {
    let image = image::open(path).ok()?;
    let thumb = image.thumbnail(THUMB_SIZE, THUMB_SIZE).to_rgba8();
    let size = [thumb.width() as usize, thumb.height() as usize];
    Some(egui::ColorImage::from_rgba_unmultiplied(
        size,
        thumb.as_raw(),
    ))
}
