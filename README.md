# Pine Script (Zed Extension)

TradingView Pine Script v6 support for the [Zed](https://zed.dev) editor.

## Features

- **Diagnostics** — parse errors and lint warnings as you type
- **Hover docs** — documentation for built-in functions and variables
- **Completions** — keywords, built-in functions, and variables

These features are provided by the [`pine-lsp`](https://github.com/nuniesmith/pine-lsp)
language server.

## Language server

The extension looks for the `pine-lsp` binary in this order:

1. A `pine-lsp` binary already on your `PATH`.
2. Otherwise, it automatically downloads the latest release from
   [`nuniesmith/pine-lsp`](https://github.com/nuniesmith/pine-lsp/releases).

No manual setup is required for the common case. If you'd rather install it
yourself (for example to track a specific version), install it onto your `PATH`:

```sh
cargo install --git https://github.com/nuniesmith/pine-lsp pine-lsp
```

After installing, restart Zed or run **Developer: Reload Extensions**.

## License

This extension is released under the [MIT License](LICENSE). The
[`pine-lsp`](https://github.com/nuniesmith/pine-lsp) language server it uses is
also MIT licensed.
