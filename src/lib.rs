use zed_extension_api::{self as zed, LanguageServerId, Result};

struct PineScriptExtension {
    cached_binary_path: Option<String>,
}

impl PineScriptExtension {
    /// Resolve the `pine-lsp` binary.
    ///
    /// Resolution order:
    /// 1. A `pine-lsp` binary already on the user's `PATH`.
    /// 2. A previously downloaded binary cached for this session.
    /// 3. The latest release downloaded from GitHub.
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // 1. Prefer a binary the user has installed on their PATH.
        if let Some(path) = worktree.which("pine-lsp") {
            return Ok(path);
        }

        // 2. Reuse a binary downloaded earlier in this session.
        if let Some(path) = &self.cached_binary_path {
            if std::fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        // 3. Download the latest release from GitHub.
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "nuniesmith/pine-lsp",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();

        let arch = match arch {
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X8664 => "x86_64",
            zed::Architecture::X86 => "x86",
        };
        let (os, ext) = match platform {
            zed::Os::Mac => ("apple-darwin", "tar.gz"),
            zed::Os::Linux => ("unknown-linux-gnu", "tar.gz"),
            zed::Os::Windows => ("pc-windows-msvc", "zip"),
        };

        let asset_name = format!("pine-lsp-{arch}-{os}.{ext}");
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "no release asset found matching {asset_name:?} in pine-lsp {}",
                    release.version
                )
            })?;

        let version_dir = format!("pine-lsp-{}", release.version);
        let binary_name = match platform {
            zed::Os::Windows => "pine-lsp.exe",
            _ => "pine-lsp",
        };
        let binary_path = format!("{version_dir}/{binary_name}");

        if !std::fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            let file_kind = match platform {
                zed::Os::Windows => zed::DownloadedFileType::Zip,
                _ => zed::DownloadedFileType::GzipTar,
            };

            zed::download_file(&asset.download_url, &version_dir, file_kind)
                .map_err(|err| format!("failed to download pine-lsp: {err}"))?;

            zed::make_file_executable(&binary_path)?;

            // Remove any stale versions we downloaded previously.
            if let Ok(entries) = std::fs::read_dir(".") {
                for entry in entries.flatten() {
                    if entry.file_name().to_str() != Some(&version_dir) {
                        std::fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for PineScriptExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(PineScriptExtension);
