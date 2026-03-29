use zed_extension_api::{self as zed, LanguageServerId, Result};

struct PineScriptExtension;

impl zed::Extension for PineScriptExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        worktree
            .which("pine-lsp")
            .map(|path| zed::Command {
                command: path,
                args: vec![],
                env: Default::default(),
            })
            .ok_or_else(|| {
                "pine-lsp not found on PATH.\n\n\
                 Install it with:\n\
                 \n\
                 cargo install --git https://github.com/nuniesmith/pine-script-zed pine-lsp\n\
                 \n\
                 Then reload Zed."
                    .into()
            })
    }
}

zed::register_extension!(PineScriptExtension);
