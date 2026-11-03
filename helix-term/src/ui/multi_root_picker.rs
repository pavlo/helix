//! File picker that searches files across multiple configured directories.
//!
//! This is intentionally kept separate from [`super::file_picker`] so that the standard picker
//! and its supporting infrastructure stay close to upstream. The traversal and picker setup
//! mirror `super::file_picker`, using the same ignore rules, `Picker`/`Injector` and `nucleo`
//! matching. If the standard picker gains new traversal options upstream, mirror them here.

use std::error::Error;
use std::path::PathBuf;

use helix_view::{theme::Style, Editor};
use tui::text::{Span, Spans};

use super::{Picker, PickerColumn};

#[derive(Debug)]
pub struct MultiRootFilePickerData {
    directory_style: Style,
}
pub type MultiRootFilePicker = Picker<PathBuf, MultiRootFilePickerData>;

/// Collect files from the given search roots. Overlapping roots are deduplicated.
fn file_picker_files(
    search_paths: &[PathBuf],
    config: &helix_view::editor::FilePickerConfig,
) -> impl Iterator<Item = PathBuf> + 'static {
    use ignore::WalkBuilder;

    let dedup_symlinks = config.deduplicate_links;
    let absolute_roots: Vec<PathBuf> = search_paths
        .iter()
        .map(|root| root.canonicalize().unwrap_or_else(|_| root.clone()))
        .collect();

    let mut walk_builder = WalkBuilder::new(&search_paths[0]);
    for path in search_paths.iter().skip(1) {
        walk_builder.add(path);
    }

    let mut seen = std::collections::HashSet::new();

    walk_builder
        .hidden(config.hidden)
        .parents(config.parents)
        .ignore(config.ignore)
        .follow_links(config.follow_symlinks)
        .git_ignore(config.git_ignore)
        .git_global(config.git_global)
        .git_exclude(config.git_exclude)
        .sort_by_file_name(|name1, name2| name1.cmp(name2))
        .max_depth(config.max_depth)
        .filter_entry(move |entry| {
            // Reuse the standard picker's filter for each root. For symlinks this deduplicates
            // links pointing into any of the search roots.
            absolute_roots
                .iter()
                .all(|root| crate::filter_picker_entry(entry, root, dedup_symlinks))
        })
        .add_custom_ignore_filename(helix_loader::config_dir().join("ignore"))
        .add_custom_ignore_filename(".helix/ignore")
        .types(super::get_excluded_types())
        .build()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.path().is_file() {
                return None;
            }
            Some(entry.into_path())
        })
        .filter(move |path| seen.insert(path.clone()))
}

pub fn multi_root_file_picker(editor: &Editor, search_paths: Vec<PathBuf>) -> MultiRootFilePicker {
    use std::time::Instant;

    let config = editor.config();
    let data = MultiRootFilePickerData {
        directory_style: editor.theme.get("ui.text.directory"),
    };

    let now = Instant::now();
    let mut files = file_picker_files(&search_paths, &config.file_picker);
    log::debug!(
        "multi-root file_picker init {:?}",
        Instant::now().duration_since(now)
    );

    let columns = [PickerColumn::new(
        "path",
        |item: &PathBuf, data: &MultiRootFilePickerData| {
            // Show the complete path (with the home directory folded to `~`) so that results
            // from different roots remain distinguishable and searchable by root name.
            let path = helix_stdx::path::fold_home_dir(item.as_path());
            let mut spans = Vec::with_capacity(3);
            if let Some(dirs) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                spans.extend([
                    Span::styled(dirs.to_string_lossy().into_owned(), data.directory_style),
                    Span::styled(std::path::MAIN_SEPARATOR_STR, data.directory_style),
                ]);
            }
            let filename = path
                .file_name()
                .expect("normalized paths can't end in `..`")
                .to_string_lossy()
                .into_owned();
            spans.push(Span::raw(filename));
            Spans::from(spans).into()
        },
    )];
    let picker = Picker::new(columns, 0, [], data, move |cx, path: &PathBuf, action| {
        if let Err(e) = cx.editor.open(path, action) {
            let err = if let Some(err) = e.source() {
                format!("{}", err)
            } else {
                format!("unable to open \"{}\"", path.display())
            };
            cx.editor.set_error(err);
        }
    })
    .with_preview(|_editor, path| Some((path.as_path().into(), None)));
    let injector = picker.injector();
    let timeout = std::time::Instant::now() + std::time::Duration::from_millis(30);

    let mut hit_timeout = false;
    for file in &mut files {
        if injector.push(file).is_err() {
            break;
        }
        if std::time::Instant::now() >= timeout {
            hit_timeout = true;
            break;
        }
    }
    if hit_timeout {
        std::thread::spawn(move || {
            for file in files {
                if injector.push(file).is_err() {
                    break;
                }
            }
        });
    }
    picker
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, File};

    use helix_view::editor::FilePickerConfig;

    use super::*;

    #[test]
    fn file_picker_files_walks_all_roots() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first");
        let second = root.path().join("second");
        create_dir_all(&first).unwrap();
        create_dir_all(&second).unwrap();
        File::create(first.join("one.txt")).unwrap();
        File::create(second.join("two.txt")).unwrap();

        let files: Vec<_> = file_picker_files(
            &[first.clone(), second.clone()],
            &FilePickerConfig::default(),
        )
        .collect();

        assert!(files.contains(&first.join("one.txt")));
        assert!(files.contains(&second.join("two.txt")));
    }

    #[test]
    fn file_picker_files_deduplicates_overlapping_roots() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("nested");
        create_dir_all(&nested).unwrap();
        let file = nested.join("file.txt");
        File::create(&file).unwrap();

        let files: Vec<_> = file_picker_files(
            &[root.path().to_path_buf(), nested.clone()],
            &FilePickerConfig::default(),
        )
        .collect();

        assert_eq!(files.iter().filter(|path| **path == file).count(), 1);
    }

    #[test]
    fn file_picker_files_ignores_vcs_directories() {
        let root = tempfile::tempdir().unwrap();
        create_dir_all(root.path().join(".git")).unwrap();
        let ignored = root.path().join(".git").join("config");
        File::create(&ignored).unwrap();

        let files: Vec<_> =
            file_picker_files(&[root.path().to_path_buf()], &FilePickerConfig::default()).collect();

        assert!(!files.contains(&ignored));
    }
}
