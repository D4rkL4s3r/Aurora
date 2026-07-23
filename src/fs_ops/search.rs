use super::listing::FileEntry;
use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs_ops::temp_sandbox;
    use std::fs;

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
}
