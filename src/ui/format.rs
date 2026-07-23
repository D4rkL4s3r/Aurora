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
