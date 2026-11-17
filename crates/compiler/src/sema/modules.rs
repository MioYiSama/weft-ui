//! Loads the import closure of an entry module.

use std::collections::HashMap;

use crate::diagnostic::Diagnostic;
use crate::loader::{SourceLoader, normalize, resolve_import};
use crate::span::{FileId, SourceMap, Span};
use crate::syntax::{ast, parse};

pub(crate) struct LoadedFile {
    pub ast: ast::File,
    /// Successfully loaded direct imports, with the span of the `import`.
    pub imports: Vec<(FileId, Span)>,
}

/// Loads `entry` and everything it imports. The returned vector is indexed by
/// [`FileId`]; it is empty if the entry module itself cannot be loaded.
pub(crate) fn load(
    entry: &str,
    loader: &mut dyn SourceLoader,
    sources: &mut SourceMap,
    diags: &mut Vec<Diagnostic>,
) -> Vec<LoadedFile> {
    let entry = normalize(entry);
    let text = match loader.load(&entry) {
        Ok(text) => text,
        Err(err) => {
            let file = sources.add(entry.clone(), String::new());
            diags.push(Diagnostic::error(
                Span::new(file, 0, 0),
                format!("cannot load entry module `{entry}`: {err}"),
            ));
            return Vec::new();
        }
    };

    let mut ids: HashMap<String, FileId> = HashMap::new();
    ids.insert(entry.clone(), sources.add(entry, text));
    let mut files: Vec<LoadedFile> = Vec::new();
    // Files are parsed in id order; imports append new ids as they are found.
    while files.len() < sources.files().len() {
        let id = FileId(crate::span::offset(files.len()));
        let (tree, mut parse_diags) = parse(id, sources.get(id).text());
        diags.append(&mut parse_diags);

        let path = sources.get(id).path().to_owned();
        let mut imports = Vec::new();
        for import in &tree.imports {
            let target = resolve_import(&path, &import.path);
            let target_id = match ids.get(&target) {
                Some(&existing) => existing,
                None => match loader.load(&target) {
                    Ok(text) => {
                        let new_id = sources.add(target.clone(), text);
                        ids.insert(target.clone(), new_id);
                        new_id
                    }
                    Err(err) => {
                        diags.push(
                            Diagnostic::error(
                                import.path_span,
                                format!("cannot load module `{target}`: {err}"),
                            )
                            .with_label("imported here"),
                        );
                        continue;
                    }
                },
            };
            if imports.iter().any(|&(existing, _)| existing == target_id) {
                diags.push(
                    Diagnostic::warning(import.span, format!("`{target}` is imported twice"))
                        .with_label("duplicate import"),
                );
                continue;
            }
            imports.push((target_id, import.span));
        }
        files.push(LoadedFile { ast: tree, imports });
    }
    files
}
