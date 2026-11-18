//! Zed extension for Weft UI: starts `weft lsp` for `.weft` files.

use zed_extension_api::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result};

struct Weft;

impl zed::Extension for Weft {
    fn new() -> Self {
        Self
    }

    /// Runs `lsp.weft.binary` from the settings if set, otherwise `weft lsp`
    /// from the worktree's `PATH`.
    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|settings| settings.binary);
        let (path, arguments, env) = match binary {
            Some(binary) => (binary.path, binary.arguments, binary.env),
            None => (None, None, None),
        };
        let command = path.or_else(|| worktree.which("weft")).ok_or_else(|| {
            "`weft` was not found on PATH. Install it from a Weft UI checkout with \
             `cargo install --path crates/cli`, or set `lsp.weft.binary.path` in Zed's settings."
                .to_owned()
        })?;
        let mut shell_env = worktree.shell_env();
        shell_env.extend(env.unwrap_or_default());
        Ok(zed::Command {
            command,
            args: arguments.unwrap_or_else(|| vec!["lsp".to_owned()]),
            env: shell_env,
        })
    }
}

zed::register_extension!(Weft);
