//! Web DOM backend: logic modules whose components create and patch DOM
//! nodes directly, without a view protocol.
//!
//! Static structure is cloned from `<template>`s; every block gets an update
//! function that re-runs only the bindings whose dirty bits are set. See
//! `@weft-ui/runtime` for the runtime half of this contract.

mod dom;

use weft_ui_codegen_js::{JsModule, Mode};
use weft_ui_compiler::ir::Program;
use weft_ui_compiler::span::FileId;

/// Import specifier of the JS runtime used by the generated modules.
pub const RUNTIME: &str = "@weft-ui/runtime";

/// Generates the Web logic module of `file`.
///
/// # Panics
///
/// Panics if `file` is not part of the program.
#[must_use]
pub fn module(program: &Program, file: FileId) -> JsModule {
    weft_ui_codegen_js::generate(program, file, RUNTIME, Mode::Dom(&dom::Dom))
}
