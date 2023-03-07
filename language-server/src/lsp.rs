use std::str::Split;

use pest::error::ErrorVariant;
use pest_meta::parser::Rule;
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        request::{GotoDeclarationParams, GotoDeclarationResponse},
        CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
        DeleteFilesParams, Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
        DidChangeWatchedFilesParams, DidOpenTextDocumentParams, DocumentChanges,
        DocumentFormattingParams, FileChangeType, FileDelete, FileEvent, GotoDefinitionParams,
        GotoDefinitionResponse, Hover, HoverContents, HoverParams, InitializedParams, Location,
        MarkedString, MessageType, OneOf, OptionalVersionedTextDocumentIdentifier, Position,
        PublishDiagnosticsParams, Range, ReferenceParams, RenameParams, TextDocumentEdit,
        TextDocumentItem, TextEdit, Url, VersionedTextDocumentIdentifier, WorkspaceEdit,
    },
    Client,
};

use crate::builtins::{get_builtin_description, BUILTINS};
use crate::helpers::{
    get_empty_diagnostics, parse_pest_grammar, Diagnostics, Documents, FindAllOccurrences,
    FindWord, IntoLocation, IntoRange, IntoRangeWithLine,
};

#[derive(Debug)]
pub struct PestLanguageServerImpl {
    pub(crate) client: Client,
    pub(crate) documents: Documents,
    pub(crate) cached_rule_identifiers: Vec<String>,
}

impl PestLanguageServerImpl {
    pub async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Pest Language Server v{}", env!("CARGO_PKG_VERSION")),
            )
            .await;
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "Pest Language Server shutting down...")
            .await;
        Ok(())
    }

    pub async fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let DidOpenTextDocumentParams { text_document } = params;
        self.client
            .log_message(MessageType::INFO, format!("Opening {}", text_document.uri))
            .await;

        if self.upsert_document(text_document).is_some() {
            self.client
                .log_message(
                    MessageType::INFO,
                    "\tReopened already tracked document.".to_string(),
                )
                .await;
        }

        let diagnostics = self.reload().await;
        self.send_diagnostics(&diagnostics).await;
    }

    pub async fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document,
            content_changes,
        } = params;
        let VersionedTextDocumentIdentifier { uri, version } = text_document;

        assert_eq!(content_changes.len(), 1);
        let change = content_changes.into_iter().next().unwrap();
        assert!(change.range.is_none());

        let updated_doc =
            TextDocumentItem::new(uri.clone(), "pest".to_owned(), version, change.text);

        if self.upsert_document(updated_doc).is_none() {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Updated untracked document {}", uri),
                )
                .await;
        }

        let diagnostics = self.reload().await;
        self.send_diagnostics(&diagnostics).await;
    }

    pub async fn did_change_watched_files(&mut self, params: DidChangeWatchedFilesParams) {
        let DidChangeWatchedFilesParams { changes } = params;
        let uris: Vec<_> = changes
            .into_iter()
            .map(|FileEvent { uri, typ }| {
                assert_eq!(typ, FileChangeType::DELETED);
                uri
            })
            .collect();

        let mut diagnostics = Diagnostics::new();

        for uri in uris {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Deleting removed document {}", uri),
                )
                .await;

            if let Some(removed) = self.remove_document(&uri) {
                let (_, empty_diagnostics) = get_empty_diagnostics((&uri, &removed));
                if diagnostics.insert(uri, empty_diagnostics).is_some() {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            "\tDuplicate URIs in event payload".to_string(),
                        )
                        .await;
                }
            } else {
                self.client
                    .log_message(
                        MessageType::INFO,
                        "\tAttempted to delete untracked document".to_string(),
                    )
                    .await;
            }
        }

        diagnostics.append(&mut self.reload().await);
        self.send_diagnostics(&diagnostics).await;
    }

    pub async fn did_delete_files(&mut self, params: DeleteFilesParams) {
        let DeleteFilesParams { files } = params;
        let mut uris = vec![];
        for FileDelete { uri } in files {
            match Url::parse(&uri) {
                Ok(uri) => uris.push(uri),
                Err(e) => {
                    self.client
                        .log_message(MessageType::INFO, format!("Failed to parse URI {}", e))
                        .await
                }
            }
        }

        let mut diagnostics = Diagnostics::new();

        self.client
            .log_message(MessageType::INFO, format!("Deleting {} files", uris.len()))
            .await;

        for uri in uris {
            let removed = self.remove_documents_in_dir(&uri);
            if !removed.is_empty() {
                for (uri, params) in removed {
                    self.client
                        .log_message(MessageType::INFO, format!("\tDeleted {}", uri))
                        .await;

                    if diagnostics.insert(uri, params).is_some() {
                        self.client
                            .log_message(
                                MessageType::INFO,
                                "\tDuplicate URIs in event payload".to_string(),
                            )
                            .await;
                    }
                }
            }
        }

        if !diagnostics.is_empty() {
            diagnostics.append(&mut self.reload().await);
            self.send_diagnostics(&diagnostics).await;
        }
    }

    pub fn completion(&mut self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let CompletionParams {
            text_document_position,
            ..
        } = params;

        let document = self
            .documents
            .get(&text_document_position.text_document.uri)
            .unwrap();

        let mut lines = document.text.lines();
        let line = lines
            .nth(text_document_position.position.line as usize)
            .unwrap_or("");
        let range = line.get_word_range_at_idx(text_document_position.position.character as usize);
        let partial_identifier = &line[range];

        let pairs = parse_pest_grammar(&document.text);

        if let Ok(pairs) = pairs {
            self.cached_rule_identifiers = Vec::new();
            self.cached_rule_identifiers
                .extend(BUILTINS.iter().map(|s| s.to_string()));

            for pair in pairs {
                if pair.as_rule() == Rule::grammar_rule {
                    let mut inner = pair.into_inner();
                    let identifier = loop {
                        if let Some(inner) = inner.next() {
                            if inner.as_rule() == Rule::identifier {
                                break Some(inner);
                            }
                        } else {
                            break None;
                        }
                    };

                    if let Some(inner) = identifier {
                        self.cached_rule_identifiers.push(inner.as_str().to_owned());
                    }
                }
            }
        }

        Ok(Some(CompletionResponse::Array(
            self.cached_rule_identifiers
                .iter()
                .filter(|i| partial_identifier.is_empty() || i.starts_with(partial_identifier))
                .map(|i| CompletionItem {
                    label: i.to_owned(),
                    kind: Some(CompletionItemKind::FIELD),
                    ..Default::default()
                })
                .collect(),
        )))
    }

    pub fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let HoverParams {
            text_document_position_params,
            ..
        } = params;
        let document = self
            .documents
            .get(&text_document_position_params.text_document.uri)
            .unwrap();

        let mut lines = document.text.lines();
        let line = lines
            .nth(text_document_position_params.position.line as usize)
            .unwrap_or("");
        let range =
            line.get_word_range_at_idx(text_document_position_params.position.character as usize);
        let word = &line[range.clone()];

        Ok(Some(
            if let Some(description) = get_builtin_description(word) {
                Hover {
                    contents: HoverContents::Scalar(MarkedString::String(description.to_owned())),
                    range: Some(range.into_lsp_range(text_document_position_params.position.line)),
                }
            } else {
                Hover {
                    contents: HoverContents::Scalar(MarkedString::String("".to_string())),
                    range: Some(range.into_lsp_range(text_document_position_params.position.line)),
                }
            },
        ))
    }

    pub fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let RenameParams {
            text_document_position,
            new_name,
            ..
        } = params;

        let document = self
            .documents
            .get(&text_document_position.text_document.uri)
            .unwrap();
        let line = document
            .text
            .lines()
            .nth(text_document_position.position.line as usize)
            .unwrap_or("");
        let old_name =
            &line[line.get_word_range_at_idx(text_document_position.position.character as usize)];

        let pairs = parse_pest_grammar(&document.text);
        let mut edits = Vec::new();

        if let Ok(pairs) = pairs {
            let spans = pairs.find_all_occurrences(old_name);

            for span in spans {
                edits.push(TextEdit {
                    range: span.into_lsp_range(),
                    new_text: new_name.clone(),
                });
            }
        }

        Ok(Some(WorkspaceEdit {
            change_annotations: None,
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document_position.text_document.uri,
                    version: Some(document.version),
                },
                edits: edits.iter().map(|edit| OneOf::Left(edit.clone())).collect(),
            }])),
        }))
    }

    pub fn goto_declaration(
        &self,
        params: GotoDeclarationParams,
    ) -> Result<Option<GotoDeclarationResponse>> {
        let GotoDeclarationParams {
            text_document_position_params,
            ..
        } = params;

        let document = self
            .documents
            .get(&text_document_position_params.text_document.uri)
            .unwrap();

        let mut lines = document.text.lines();
        let line = lines
            .nth(text_document_position_params.position.line as usize)
            .unwrap_or("");

        let range =
            line.get_word_range_at_idx(text_document_position_params.position.character as usize);
        let identifier = &line[range];

        let pairs = parse_pest_grammar(&document.text);
        let mut definition: Option<Range> = None;

        if let Ok(pairs) = pairs {
            for pair in pairs {
                if pair.as_rule() == Rule::grammar_rule {
                    let mut inner = pair.into_inner();
                    let inner = inner.next().unwrap();

                    if inner.as_str() == identifier {
                        definition = Some(inner.as_span().into_lsp_range());
                    }
                }

                if let Some(definition) = definition {
                    return Ok(Some(GotoDeclarationResponse::Scalar(Location {
                        uri: text_document_position_params.text_document.uri,
                        range: definition,
                    })));
                }
            }
        }

        Ok(None)
    }

    pub fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let GotoDefinitionParams {
            text_document_position_params,
            ..
        } = params;

        let document = self
            .documents
            .get(&text_document_position_params.text_document.uri)
            .unwrap();

        let mut lines = document.text.lines();
        let line = lines
            .nth(text_document_position_params.position.line as usize)
            .unwrap_or("");
        let range =
            line.get_word_range_at_idx(text_document_position_params.position.character as usize);
        let identifier = &line[range];

        let pairs = parse_pest_grammar(&document.text);
        let mut definition: Option<Range> = None;

        if let Ok(pairs) = pairs {
            for pair in pairs {
                if pair.as_rule() == Rule::grammar_rule {
                    let mut inner = pair.into_inner();
                    let inner = inner.next().unwrap();

                    if inner.as_str() == identifier {
                        definition = Some(inner.as_span().into_lsp_range());
                    }
                }

                if let Some(definition) = definition {
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                        uri: text_document_position_params.text_document.uri,
                        range: definition,
                    })));
                }
            }
        }

        Ok(None)
    }

    pub fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let ReferenceParams {
            text_document_position,
            ..
        } = params;

        let document = self
            .documents
            .get(&text_document_position.text_document.uri)
            .unwrap();

        let mut lines = document.text.lines();
        let line = lines
            .nth(text_document_position.position.line as usize)
            .unwrap_or("");
        let range = line.get_word_range_at_idx(text_document_position.position.character as usize);
        let identifier = &line[range];

        let pairs = parse_pest_grammar(&document.text);
        let mut references: Vec<Location> = Vec::new();

        if let Ok(pairs) = pairs {
            let spans = pairs.find_all_occurrences(identifier);

            for span in spans {
                references.push(span.into_lsp_location(&document.uri));
            }
        }

        Ok(Some(references))
    }

    pub fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let DocumentFormattingParams { text_document, .. } = params;

        let document = self.documents.get(&text_document.uri).unwrap();
        let input = document.text.as_str();

        let fmt = pest_fmt::Formatter::new(input);
        if let Ok(formatted) = fmt.format() {
            let lines = document.text.lines();
            let last_line = lines.clone().last().unwrap_or("");
            let range = Range::new(
                Position::new(0, 0),
                Position::new(lines.count() as u32, last_line.len() as u32),
            );
            return Ok(Some(vec![TextEdit::new(range, formatted)]));
        }

        Ok(None)
    }
}

impl PestLanguageServerImpl {
    async fn reload(&mut self) -> Diagnostics {
        self.client
            .log_message(MessageType::INFO, "Reloading all diagnostics".to_string())
            .await;
        let mut diagnostics = Diagnostics::new();

        for (url, document) in &self.documents {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("\tReloading diagnostics for {}", url),
                )
                .await;
            let res = parse_pest_grammar(document.text.as_str());

            if let Err(errors) = res {
                diagnostics.insert(
                    url.clone(),
                    PublishDiagnosticsParams::new(
                        url.clone(),
                        errors
                            .iter()
                            .map(|e| {
                                Diagnostic::new(
                                    e.line_col.clone().into_lsp_range(),
                                    Some(DiagnosticSeverity::ERROR),
                                    None,
                                    Some("Pest Language Server".to_owned()),
                                    match &e.variant {
                                        ErrorVariant::ParsingError {
                                            positives,
                                            negatives,
                                        } => {
                                            let mut message = "Parsing error".to_owned();
                                            if !positives.is_empty() {
                                                message.push_str(" (expected ");
                                                message.push_str(
                                                    positives
                                                        .iter()
                                                        .map(|s| format!("\"{:#?}\"", s))
                                                        .collect::<Vec<String>>()
                                                        .join(", ")
                                                        .as_str(),
                                                );
                                                message.push(')');
                                            }

                                            if !negatives.is_empty() {
                                                message.push_str(" (unexpected ");
                                                message.push_str(
                                                    negatives
                                                        .iter()
                                                        .map(|s| format!("\"{:#?}\"", s))
                                                        .collect::<Vec<String>>()
                                                        .join(", ")
                                                        .as_str(),
                                                );
                                                message.push(')');
                                            }

                                            message
                                        }
                                        ErrorVariant::CustomError { message } => {
                                            let mut c = message.chars();
                                            match c.next() {
                                                None => String::new(),
                                                Some(f) => {
                                                    f.to_uppercase().collect::<String>()
                                                        + c.as_str()
                                                }
                                            }
                                        }
                                    },
                                    None,
                                    None,
                                )
                            })
                            .collect(),
                        Some(document.version),
                    ),
                );
            } else {
                let (_, empty_diagnostics) = get_empty_diagnostics((url, document));
                diagnostics.insert(url.clone(), empty_diagnostics);
            }
        }

        diagnostics
    }

    fn upsert_document(&mut self, doc: TextDocumentItem) -> Option<TextDocumentItem> {
        self.documents.insert(doc.uri.clone(), doc)
    }

    fn remove_document(&mut self, uri: &Url) -> Option<TextDocumentItem> {
        self.documents.remove(uri)
    }

    fn remove_documents_in_dir(&mut self, dir: &Url) -> Diagnostics {
        let (in_dir, not_in_dir): (Documents, Documents) =
            self.documents.clone().into_iter().partition(|(uri, _)| {
                let maybe_segments = dir.path_segments().zip(uri.path_segments());
                let compare_paths = |(l, r): (Split<_>, Split<_>)| l.zip(r).all(|(l, r)| l == r);
                maybe_segments.map_or(false, compare_paths)
            });

        self.documents = not_in_dir;
        in_dir.iter().map(get_empty_diagnostics).collect()
    }

    async fn send_diagnostics(&self, diagnostics: &Diagnostics) {
        for PublishDiagnosticsParams {
            uri,
            diagnostics,
            version,
        } in diagnostics.values()
        {
            self.client
                .publish_diagnostics(uri.clone(), diagnostics.clone(), *version)
                .await;
        }
    }
}
