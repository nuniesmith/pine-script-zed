# Pine Script (Zed Extension)

TradingView Pine Script v6 support for the [Zed](https://zed.dev) editor.

## Features

- **Diagnostics** — parse errors and lint warnings as you type
- **Hover docs** — documentation for built-in functions and variables
- **Completions** — keywords, built-in functions, and variables

## Requirements

This extension requires the `pine-lsp` language server binary on your `PATH`.

### Install pine-lsp

```sh
cargo install --git https://github.com/nuniesmith/pine pine-lsp
```

Or build from source:

```sh
git clone https://github.com/nuniesmith/pine.git
cd pine/src/pine-tools
cargo install --path pine-lsp
```

After installing, restart Zed or run **Developer: Reload Extensions**.

## License

This extension code is released under the [MIT License](LICENSE).

The `pine-lsp` language server is licensed under the
[Mozilla Public License 2.0](https://mozilla.org/MPL/2.0/).