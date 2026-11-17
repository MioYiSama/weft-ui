//! TypeScript host bindings with the Web DOM backend.
//!
//! Each `.weft` file becomes one ES module plus a `.d.ts`. A module imports
//! what it uses from its direct imports and re-exports every symbol of its
//! import closure, so hosts only reference the entry module.
//!
//! The WeChat mini program backend ([`crate::codegen::miniprogram`]) shares
//! the host binding and swaps the DOM create functions for data blocks.

mod blocks;
mod dom;
mod expr;
mod host;
mod js;
mod wx;
mod wxml;

use std::collections::{BTreeMap, BTreeSet};

use crate::ir::{ComponentId, EventId, ModelId, Module, NodeKind, ParamKind, Program, Type};
use crate::span::FileId;

use js::Code;

/// Options for [`generate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebOptions {
    /// Import specifier of the JS runtime package.
    pub runtime: String,
}

impl Default for WebOptions {
    fn default() -> Self {
        Self {
            runtime: "@weft-ui/runtime".to_owned(),
        }
    }
}

/// Generated host module for one `.weft` file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsModule {
    /// ES module source.
    pub code: String,
    /// TypeScript declarations for `code`.
    pub dts: String,
}

/// Generates the host module of the program's entry file.
#[must_use]
pub fn generate(program: &Program, options: &WebOptions) -> JsModule {
    generate_module(program, program.entry, options)
}

/// Generates the host module of `file`, which must belong to the program's
/// import closure. Its imports are specified relative to `file`, so the
/// modules of a whole closure can be served side by side.
///
/// # Panics
///
/// Panics if `file` is not part of the program.
#[must_use]
pub fn generate_module(program: &Program, file: FileId, options: &WebOptions) -> JsModule {
    backend_module(program, file, &options.runtime, Backend::Dom)
}

/// Rendering backend of a generated host module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::codegen) enum Backend {
    /// Browser DOM; modules import each other as `.weft` files.
    Dom,
    /// WeChat mini program; modules import each other as `.js` files.
    Wx,
}

/// Generates the host module of `file` for `backend`, importing the runtime
/// from `runtime`.
pub(in crate::codegen) fn backend_module(
    program: &Program,
    file: FileId,
    runtime: &str,
    backend: Backend,
) -> JsModule {
    let links = Links::new(program, file, backend);
    JsModule {
        code: code(program, &links, runtime),
        dts: dts(program, &links),
    }
}

/// WXML templates of every component of the program (mini program backend).
pub(in crate::codegen) fn wxml_templates(program: &Program) -> String {
    wxml::templates(program)
}

/// Mini program path of the JS module generated for the `.weft` file `path`.
pub(in crate::codegen) fn wx_module_path(path: &str) -> String {
    format!("{}.js", path.strip_suffix(".weft").unwrap_or(path))
}

/// Relative import specifier between two module paths.
pub(in crate::codegen) fn relative_specifier(from: &str, to: &str) -> String {
    js::relative_specifier(from, to)
}

/// Double-quoted JS string literal.
pub(in crate::codegen) fn js_string(value: &str) -> String {
    js::string(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Symbol {
    Model(ModelId),
    Event(EventId),
    Component(ComponentId),
}

impl Symbol {
    fn name(self, program: &Program) -> &str {
        match self {
            Symbol::Model(id) => &program.model(id).name,
            Symbol::Event(id) => &program.event(id).name,
            Symbol::Component(id) => &program.component(id).name,
        }
    }

    fn file(self, program: &Program) -> FileId {
        match self {
            Symbol::Model(id) => program.model(id).file,
            Symbol::Event(id) => program.event(id).file,
            Symbol::Component(id) => program.component(id).file,
        }
    }
}

/// How the entry module reaches symbols declared in other files.
struct Links<'p> {
    program: &'p Program,
    entry: &'p Module,
    /// Import closure of each direct import, including itself.
    closures: Vec<(&'p Module, BTreeSet<FileId>)>,
    backend: Backend,
}

impl<'p> Links<'p> {
    fn new(program: &'p Program, file: FileId, backend: Backend) -> Self {
        let module = |file: FileId| {
            program
                .modules
                .iter()
                .find(|m| m.file == file)
                .expect("every file of the program has a module")
        };
        let entry = module(file);
        let closures = entry
            .imports
            .iter()
            .map(|&file| {
                let mut seen = BTreeSet::new();
                let mut stack = vec![file];
                while let Some(next) = stack.pop() {
                    if seen.insert(next) {
                        stack.extend(&module(next).imports);
                    }
                }
                (module(file), seen)
            })
            .collect();
        Self {
            program,
            entry,
            closures,
            backend,
        }
    }

    /// The direct import whose module exports `symbol`.
    fn provider(&self, symbol: Symbol) -> &'p Module {
        let file = symbol.file(self.program);
        self.closures
            .iter()
            .find(|(_, closure)| closure.contains(&file))
            .map(|(module, _)| *module)
            .expect("symbols of other files are reachable through a direct import")
    }

    fn specifier(&self, module: &Module) -> String {
        let specifier = js::relative_specifier(&self.entry.path, &module.path);
        js::string(&match self.backend {
            Backend::Dom => specifier,
            Backend::Wx => wx_module_path(&specifier),
        })
    }

    fn is_own(&self, symbol: Symbol) -> bool {
        symbol.file(self.program) == self.entry.file
    }

    /// Symbols of other files grouped by the import that provides them.
    fn group(&self, symbols: impl IntoIterator<Item = Symbol>) -> BTreeMap<FileId, Vec<Symbol>> {
        let mut groups: BTreeMap<FileId, Vec<Symbol>> = BTreeMap::new();
        for symbol in symbols {
            if !self.is_own(symbol) {
                groups
                    .entry(self.provider(symbol).file)
                    .or_default()
                    .push(symbol);
            }
        }
        groups
    }

    fn module(&self, file: FileId) -> &'p Module {
        self.closures
            .iter()
            .map(|(m, _)| *m)
            .find(|m| m.file == file)
            .expect("groups only contain direct imports")
    }

    /// Every symbol of other files in this module's import closure, for
    /// re-export.
    fn foreign(&self) -> impl Iterator<Item = Symbol> + '_ {
        let program = self.program;
        let models = (0..program.models.len()).map(|i| Symbol::Model(ModelId(offset(i))));
        let events = (0..program.events.len()).map(|i| Symbol::Event(EventId(offset(i))));
        let components =
            (0..program.components.len()).map(|i| Symbol::Component(ComponentId(offset(i))));
        models.chain(events).chain(components).filter(|&s| {
            let file = s.file(program);
            !self.is_own(s) && self.closures.iter().any(|(_, c)| c.contains(&file))
        })
    }
}

fn offset(index: usize) -> u32 {
    crate::span::offset(index)
}

fn models_in(ty: &Type, out: &mut BTreeSet<Symbol>) {
    match ty {
        Type::Model(id) => {
            out.insert(Symbol::Model(*id));
        }
        Type::Array(inner) | Type::Optional(inner) => models_in(inner, out),
        _ => {}
    }
}

/// Models named by the types of the entry's models and component parameters.
fn used_models(program: &Program, entry: &Module, out: &mut BTreeSet<Symbol>) {
    for &id in &entry.models {
        for field in &program.model(id).fields {
            models_in(&field.ty, out);
        }
    }
    for &id in &entry.components {
        for param in &program.component(id).params {
            if let ParamKind::Value(ty) = &param.kind {
                models_in(ty, out);
            }
        }
    }
}

fn code(program: &Program, links: &Links<'_>, runtime: &str) -> String {
    let entry = links.entry;
    let mut out = Code::default();
    out.line(format!("import * as $ from {};", js::string(runtime)));

    // Imports: models for converters, create functions of used components.
    let mut used = BTreeSet::new();
    used_models(program, entry, &mut used);
    let mut callees = BTreeSet::new();
    for &id in &entry.components {
        for node in &program.component(id).nodes {
            if let NodeKind::Component { component, .. } = node.kind {
                callees.insert(Symbol::Component(component));
            }
        }
    }
    let mut imports: BTreeMap<FileId, Vec<String>> = BTreeMap::new();
    for (file, symbols) in links.group(used) {
        imports
            .entry(file)
            .or_default()
            .extend(symbols.iter().map(|s| {
                let name = s.name(program);
                js::aliased(name, &js::binding(name))
            }));
    }
    for (file, symbols) in links.group(callees) {
        imports
            .entry(file)
            .or_default()
            .extend(symbols.iter().map(|s| format!("{}$c", s.name(program))));
    }
    for (file, names) in imports {
        out.line(format!(
            "import {{ {} }} from {};",
            names.join(", "),
            links.specifier(links.module(file))
        ));
    }
    // Events have no runtime value.
    let reexports = links.foreign().filter(|s| !matches!(s, Symbol::Event(_)));
    for (file, symbols) in links.group(reexports) {
        let names: Vec<&str> = symbols.iter().map(|s| s.name(program)).collect();
        out.line(format!(
            "export {{ {} }} from {};",
            names.join(", "),
            links.specifier(links.module(file))
        ));
    }

    let mut converters = host::Converters::default();
    let mut models = Code::default();
    for &id in &entry.models {
        models.line("");
        host::model_class(program, id, &mut converters, &mut models);
    }
    let mut consts = blocks::Consts::default();
    let mut components = Code::default();
    for &id in &entry.components {
        components.line("");
        match links.backend {
            Backend::Dom => dom::component(program, id, &mut consts, &mut components),
            Backend::Wx => wx::component(program, id, &mut consts, &mut components),
        }
        components.line("");
        host::app_class(program, id, links.backend, &mut converters, &mut components);
    }

    out.append(models);
    out.line("");
    converters.write(&mut out);
    consts.write(&mut out);
    out.append(components);
    let aliases = own_aliases(program, entry, false);
    if !aliases.is_empty() {
        out.line("");
        out.line(format!("export {{ {} }};", aliases.join(", ")));
    }
    let mut text = String::new();
    out.write_to(&mut text);
    text
}

/// `local as name` for own declarations whose names are reserved words.
fn own_aliases(program: &Program, entry: &Module, with_events: bool) -> Vec<String> {
    let models = entry.models.iter().map(|&id| &program.model(id).name);
    let events = entry
        .events
        .iter()
        .filter(|_| with_events)
        .map(|&id| &program.event(id).name);
    let components = entry
        .components
        .iter()
        .map(|&id| &program.component(id).name);
    models
        .chain(events)
        .chain(components)
        .filter_map(|name| {
            let local = js::binding(name);
            (local != *name).then(|| format!("{local} as {name}"))
        })
        .collect()
}

fn dts(program: &Program, links: &Links<'_>) -> String {
    let entry = links.entry;
    let mut out = Code::default();

    let mut used = BTreeSet::new();
    used_models(program, entry, &mut used);
    for &id in &entry.events {
        for param in &program.event(id).params {
            models_in(&param.ty, &mut used);
        }
    }
    for &id in &entry.components {
        used.extend(
            program
                .component(id)
                .reachable_events
                .iter()
                .map(|&e| Symbol::Event(e)),
        );
    }
    for (file, symbols) in links.group(used) {
        let names: Vec<String> = symbols
            .iter()
            .map(|s| {
                let name = s.name(program);
                js::aliased(name, &js::binding(name))
            })
            .collect();
        out.line(format!(
            "import type {{ {} }} from {};",
            names.join(", "),
            links.specifier(links.module(file))
        ));
    }
    for (file, symbols) in links.group(links.foreign()) {
        let specifier = links.specifier(links.module(file));
        let (classes, types): (Vec<Symbol>, Vec<Symbol>) = symbols
            .into_iter()
            .partition(|s| matches!(s, Symbol::Component(_)));
        let names = |symbols: &[Symbol]| {
            symbols
                .iter()
                .map(|s| s.name(program))
                .collect::<Vec<_>>()
                .join(", ")
        };
        if !types.is_empty() {
            out.line(format!(
                "export type {{ {} }} from {specifier};",
                names(&types)
            ));
        }
        if !classes.is_empty() {
            out.line(format!(
                "export {{ {} }} from {specifier};",
                names(&classes)
            ));
        }
    }

    for &id in &entry.models {
        out.line("");
        host::model_dts(program, id, &mut out);
    }
    if !entry.events.is_empty() {
        out.line("");
    }
    for &id in &entry.events {
        host::event_dts(program, id, &mut out);
    }
    for &id in &entry.components {
        out.line("");
        host::app_dts(program, id, links.backend, &mut out);
    }
    let aliases = own_aliases(program, entry, true);
    if !aliases.is_empty() {
        out.line("");
        out.line(format!("export {{ {} }};", aliases.join(", ")));
    }
    let mut text = String::new();
    out.write_to(&mut text);
    text
}
