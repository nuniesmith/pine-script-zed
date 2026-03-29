use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::ast::LintSeverity;
use crate::builtins;
use crate::line_index::LineIndex;
use crate::linter;
use crate::parser;

pub struct PineBackend {
    pub client: Client,
    pub documents: DashMap<Url, String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for PineBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), "(".into()]),
                    ..Default::default()
                }),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "pine-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Pine LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.documents.insert(uri.clone(), text.clone());
        self.on_change(uri, text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        // We use FULL sync, so take the last (and only) content change.
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            self.documents.insert(uri.clone(), text.clone());
            self.on_change(uri, text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(entry) = self.documents.get(&uri) {
            let text = entry.value().clone();
            self.on_change(uri, text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(entry) = self.documents.get(&uri) else {
            return Ok(None);
        };
        let text = entry.value();
        let line_index = LineIndex::new(text);

        let Some(offset) = line_index.offset(position) else {
            return Ok(None);
        };

        let Some(word) = word_at_offset(text, offset) else {
            return Ok(None);
        };

        // Check if the word (possibly with a namespace prefix like "ta.sma") is
        // a built-in function.
        if let Some(func) = builtins::lookup_function(&word) {
            let params_doc: Vec<String> = func
                .params
                .iter()
                .map(|(name, ty)| format!("- `{name}`: {ty}"))
                .collect();
            let mut doc = format!("### `{}`\n\n{}\n", func.name, func.doc);
            if !params_doc.is_empty() {
                doc.push_str("\n**Parameters:**\n");
                doc.push_str(&params_doc.join("\n"));
                doc.push('\n');
            }
            doc.push_str(&format!("\n**Returns:** `{}`", func.returns));
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                }),
                range: None,
            }));
        }

        // Check if it's a built-in variable / constant.
        if let Some(var) = builtins::lookup_variable(&word) {
            let doc = format!(
                "### `{}`\n\n{}\n\n**Type:** `{}`",
                var.name, var.doc, var.type_hint
            );
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                }),
                range: None,
            }));
        }

        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let mut items: Vec<CompletionItem> = Vec::new();

        // Keywords
        for kw in builtins::all_keywords() {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("keyword".into()),
                ..Default::default()
            });
        }

        // Built-in functions
        for func in builtins::all_functions() {
            items.push(CompletionItem {
                label: func.name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("→ {}", func.returns)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: func.doc.to_string(),
                })),
                ..Default::default()
            });
        }

        // Built-in variables
        for var in builtins::all_variables() {
            items.push(CompletionItem {
                label: var.name.to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some(var.type_hint.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: var.doc.to_string(),
                })),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }
}

// ── Helper methods ────────────────────────────────────────────────────────────

impl PineBackend {
    async fn on_change(&self, uri: Url, text: String) {
        let line_index = LineIndex::new(&text);
        let parse_result = parser::parse_script(&text);

        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        // Convert parse errors to diagnostics
        for err in &parse_result.errors {
            diagnostics.push(Diagnostic {
                range: line_index.range(&err.span),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("pine-lsp".into()),
                message: err.message.clone(),
                ..Default::default()
            });
        }

        // Run linter if we got a script
        if let Some(ref script) = parse_result.script {
            let lint_diags = linter::lint(script, &text);
            for ld in lint_diags {
                let severity = match ld.severity {
                    LintSeverity::Error => DiagnosticSeverity::ERROR,
                    LintSeverity::Warning => DiagnosticSeverity::WARNING,
                    LintSeverity::Info => DiagnosticSeverity::INFORMATION,
                    LintSeverity::Hint => DiagnosticSeverity::HINT,
                };
                diagnostics.push(Diagnostic {
                    range: line_index.range(&ld.span),
                    severity: Some(severity),
                    source: Some("pine-lsp".into()),
                    message: ld.message,
                    ..Default::default()
                });
            }
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

// ── Utility functions ─────────────────────────────────────────────────────────

/// Extract the "word" at a given byte offset in the source text.
///
/// A word is defined as a contiguous sequence of alphanumeric characters,
/// underscores, or dots (to support namespaced names like `ta.sma`).
fn word_at_offset(source: &str, offset: usize) -> Option<String> {
    if offset >= source.len() {
        return None;
    }

    let bytes = source.as_bytes();

    // Check the character at the offset is a valid word character.
    if !is_word_char(bytes[offset]) {
        return None;
    }

    // Walk backwards to find the start of the word.
    let mut start = offset;
    while start > 0 && is_word_char(bytes[start - 1]) {
        start -= 1;
    }

    // Walk forwards to find the end of the word.
    let mut end = offset;
    while end < bytes.len() && is_word_char(bytes[end]) {
        end += 1;
    }

    let word = &source[start..end];

    // Strip leading/trailing dots that may have been captured.
    let word = word.trim_matches('.');

    if word.is_empty() {
        None
    } else {
        Some(word.to_string())
    }
}

#[inline]
fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'.'
}
