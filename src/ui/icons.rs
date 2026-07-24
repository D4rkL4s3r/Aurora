//! Vraies icônes système : extraction `SHGetFileInfo` → conversion GDI →
//! texture egui, avec cache par extension. Module volontairement isolé :
//! tout échec d'extraction retombe sur l'icône emoji de `ui::format`.

use crate::fs_ops::FileEntry;
use std::collections::HashMap;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits,
    GetObjectW, ReleaseDC,
};
use windows::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL};
use windows::Win32::UI::Shell::{
    SHGFI_ICON, SHGFI_LARGEICON, SHGFI_USEFILEATTRIBUTES, SHFILEINFOW, SHGetFileInfoW,
};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};
use windows::core::PCWSTR;

/// Extensions dont l'icône dépend du fichier lui-même, pas de son extension.
fn has_own_icon(ext: &str) -> bool {
    matches!(ext, "exe" | "lnk" | "ico")
}

#[derive(Default)]
pub(crate) struct IconCache {
    /// `None` mémorise aussi les échecs : on ne retente pas à chaque frame.
    cache: HashMap<String, Option<egui::TextureHandle>>,
}

impl IconCache {
    /// Clé de cache : les dossiers partagent une icône, les .exe/.lnk/.ico
    /// ont la leur (clé = chemin), les autres celle de leur extension.
    fn cache_key(entry: &FileEntry) -> String {
        if entry.is_dir {
            return "<dir>".to_owned();
        }
        match entry.extension.as_deref().map(str::to_lowercase) {
            Some(ext) if has_own_icon(&ext) => entry.path.display().to_string(),
            Some(ext) => format!(".{ext}"),
            None => "<none>".to_owned(),
        }
    }

    pub(crate) fn get(
        &mut self,
        ctx: &egui::Context,
        entry: &FileEntry,
    ) -> Option<egui::TextureHandle> {
        let key = Self::cache_key(entry);
        self.cache
            .entry(key.clone())
            .or_insert_with(|| {
                load_system_icon(entry)
                    .map(|image| ctx.load_texture(key, image, egui::TextureOptions::LINEAR))
            })
            .clone()
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Demande l'icône au Shell. Pour une extension, on passe un nom factice avec
/// `SHGFI_USEFILEATTRIBUTES` : aucun accès disque. Pour les fichiers à icône
/// propre, le vrai chemin.
fn load_system_icon(entry: &FileEntry) -> Option<egui::ColorImage> {
    let by_attributes = SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES;
    let (path, attrs, flags) = if entry.is_dir {
        (wide("dossier"), FILE_ATTRIBUTE_DIRECTORY, by_attributes)
    } else {
        match entry.extension.as_deref().map(str::to_lowercase) {
            Some(ext) if has_own_icon(&ext) => (
                wide(&entry.path.display().to_string()),
                FILE_ATTRIBUTE_NORMAL,
                SHGFI_ICON | SHGFI_LARGEICON,
            ),
            Some(ext) => (
                wide(&format!("fichier.{ext}")),
                FILE_ATTRIBUTE_NORMAL,
                by_attributes,
            ),
            None => (wide("fichier"), FILE_ATTRIBUTE_NORMAL, by_attributes),
        }
    };
    unsafe {
        let mut info = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(path.as_ptr()),
            attrs,
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags,
        );
        if result == 0 || info.hIcon.is_invalid() {
            return None;
        }
        let image = hicon_to_image(info.hIcon);
        let _ = DestroyIcon(info.hIcon);
        image
    }
}

/// HICON → pixels RGBA via GetDIBits (32 bits, lignes de haut en bas).
unsafe fn hicon_to_image(hicon: HICON) -> Option<egui::ColorImage> {
    unsafe {
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(hicon, &mut icon_info).is_err() {
            return None;
        }
        let color = icon_info.hbmColor;
        let mask = icon_info.hbmMask;

        let image = (|| {
            if color.is_invalid() {
                return None; // icône monochrome : on laisse le repli emoji
            }
            let mut bmp = BITMAP::default();
            if GetObjectW(
                color.into(),
                std::mem::size_of::<BITMAP>() as i32,
                Some((&mut bmp as *mut BITMAP).cast()),
            ) == 0
            {
                return None;
            }
            let (w, h) = (bmp.bmWidth, bmp.bmHeight);
            if w <= 0 || h <= 0 {
                return None;
            }
            let mut header = BITMAPINFO::default();
            header.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            header.bmiHeader.biWidth = w;
            header.bmiHeader.biHeight = -h; // négatif : première ligne en haut
            header.bmiHeader.biPlanes = 1;
            header.bmiHeader.biBitCount = 32;
            header.bmiHeader.biCompression = BI_RGB.0;

            let hdc = GetDC(None);
            let mut pixels = vec![0u8; (w * h * 4) as usize];
            let lines = GetDIBits(
                hdc,
                color,
                0,
                h as u32,
                Some(pixels.as_mut_ptr().cast()),
                &mut header,
                DIB_RGB_COLORS,
            );
            ReleaseDC(None, hdc);
            if lines == 0 {
                return None;
            }
            for px in pixels.chunks_exact_mut(4) {
                px.swap(0, 2); // BGRA → RGBA
            }
            // Icône sans canal alpha : tout opaque plutôt que tout invisible.
            if pixels.chunks_exact(4).all(|px| px[3] == 0) {
                for px in pixels.chunks_exact_mut(4) {
                    px[3] = 255;
                }
            }
            Some(egui::ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize],
                &pixels,
            ))
        })();

        let _ = DeleteObject(color.into());
        let _ = DeleteObject(mask.into());
        image
    }
}
