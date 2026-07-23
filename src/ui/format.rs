use crate::fs_ops::FileEntry;
use std::time::SystemTime;

pub(crate) fn quick_access_icon(name: &str) -> &'static str {
    match name {
        "Bureau" => "🖥",
        "Documents" => "📄",
        "Téléchargements" => "⬇",
        "Images" => "🖼",
        "Musique" => "🎵",
        "Vidéos" => "🎬",
        _ => "⭐",
    }
}

pub(crate) fn entry_icon(entry: &FileEntry) -> &'static str {
    if entry.is_dir {
        return "📁";
    }
    let ext = entry.extension.as_deref().map(|e| e.to_lowercase());
    match ext.as_deref() {
        Some(
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tif" | "tiff",
        ) => "🖼",
        Some("mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv" | "m4v") => "🎬",
        Some("mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" | "opus") => "🎵",
        Some("zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "cab") => "🗜",
        Some(
            "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "c" | "cpp" | "h" | "hpp" | "java"
            | "cs" | "go" | "rb" | "php" | "html" | "css" | "json" | "toml" | "yaml" | "yml"
            | "xml" | "sh" | "ps1" | "bat" | "cmd" | "sql" | "md",
        ) => "📝",
        Some("exe" | "msi" | "lnk" | "dll") => "⚙",
        Some("pdf") => "📕",
        Some("xls" | "xlsx" | "ods" | "csv") => "📊",
        Some("ppt" | "pptx" | "odp") => "📽",
        _ => "📄",
    }
}

/// Couleur de la tuile d'icône en vue grille, par catégorie de fichier.
pub(crate) fn entry_tile_color(entry: &FileEntry) -> egui::Color32 {
    use egui::Color32;
    if entry.is_dir {
        return Color32::from_rgb(0xd8, 0xa8, 0x3a);
    }
    let ext = entry.extension.as_deref().map(|e| e.to_lowercase());
    match ext.as_deref() {
        Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tif" | "tiff") => {
            Color32::from_rgb(0x2f, 0x9e, 0x6b)
        }
        Some("mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv" | "m4v") => {
            Color32::from_rgb(0x3a, 0x6e, 0xd8)
        }
        Some("mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" | "opus") => {
            Color32::from_rgb(0x8a, 0x4f, 0xd3)
        }
        Some("zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "cab") => {
            Color32::from_rgb(0x6b, 0x72, 0x80)
        }
        Some(
            "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "c" | "cpp" | "h" | "hpp" | "java" | "cs"
            | "go" | "rb" | "php" | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "xml"
            | "sh" | "ps1" | "bat" | "cmd" | "sql" | "md",
        ) => Color32::from_rgb(0xd8, 0x6e, 0x3a),
        Some("exe" | "msi" | "lnk" | "dll") => Color32::from_rgb(0x4a, 0x54, 0x66),
        Some("pdf") => Color32::from_rgb(0xc4, 0x3d, 0x3d),
        _ => Color32::from_rgb(0x55, 0x5c, 0x68),
    }
}

pub(crate) fn type_label(entry: &FileEntry) -> String {
    if entry.is_dir {
        "Dossier".to_owned()
    } else {
        match &entry.extension {
            Some(ext) => format!("Fichier {}", ext.to_uppercase()),
            None => "Fichier".to_owned(),
        }
    }
}

pub(crate) fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["o", "Ko", "Mo", "Go", "To"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} o")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub(crate) fn format_date(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.format("%d/%m/%Y %H:%M").to_string()
}
