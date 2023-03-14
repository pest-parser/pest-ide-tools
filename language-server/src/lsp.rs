use std::{collections::BTreeMap, str::Split};

use pest_meta::{parser, validator};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        request::{GotoDeclarationParams, GotoDeclarationResponse},
        CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
        ConfigurationItem, DeleteFilesParams, Diagnostic, DiagnosticSeverity,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
        DidOpenTextDocumentParams, DocumentChanges, DocumentFormattingParams, FileChangeType,
        FileDelete, FileEvent, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
        HoverParams, InitializedParams, Location, MarkedString, MessageType, OneOf,
        OptionalVersionedTextDocumentIdentifier, Position, PublishDiagnosticsParams, Range,
        ReferenceParams, RenameParams, TextDocumentEdit, TextDocumentItem, TextEdit, Url,
        VersionedTextDocumentIdentifier, WorkspaceEdit,
    },
    Client,
};

use crate::{
    analysis::Analysis,
    config::Config,
    helpers::{
        create_empty_diagnostics, Diagnostics, Documents, FindWordRange, IntoDiagnostics,
        IntoRangeWithLine,
    },
};
use crate::{builtins::get_builtin_description, update_checker::check_for_updates};

#[derive(Debug)]
pub struct PestLanguageServerImpl {
    pub client: Client,
    pub documents: Documents,
    pub analyses: BTreeMap<Url, Analysis>,
    pub config: Config,
}

impl PestLanguageServerImpl {
    pub async fn initialized(&mut self, _: InitializedParams) {
        let config_items = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: Some("pestIdeTools".to_string()),
            }])
            .await;

        let mut updated_config = false;

        if let Ok(config_items) = config_items {
            if let Some(config) = config_items.into_iter().next() {
                if let Ok(config) = serde_json::from_value(config) {
                    self.config = config;
                    updated_config = true;
                }
            }
        }

        if !updated_config {
            self.client
                .log_message(
                    MessageType::ERROR,
                    "Failed to retrieve configuration from client.",
                )
                .await;
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!("Pest Language Server v{}", env!("CARGO_PKG_VERSION")),
            )
            .await;

        if self.config.check_for_updates {
            if let Some(new_version) = check_for_updates().await {
                self.client
                    .show_message(
                        MessageType::INFO,
                        format!(
                            "A new version of the Pest Language Server is available: v{}",
                            new_version
                        ),
                    )
                    .await;
            }
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "Pest Language Server shutting down :)")
            .await;
        Ok(())
    }

    pub async fn did_change_configuration(&mut self, params: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, format!("{:#?}", params))
            .await;
        if let Some(config) = params.settings.get("pestIdeTools") {
            if let Ok(config) = serde_json::from_value(config.clone()) {
                self.config = config;
                self.client
                    .log_message(
                        MessageType::INFO,
                        "Updated configuration from client.".to_string(),
                    )
                    .await;
            }
        }
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
        self.send_diagnostics(diagnostics).await;
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
        self.send_diagnostics(diagnostics).await;
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
                let (_, empty_diagnostics) = create_empty_diagnostics((&uri, &removed));
                if diagnostics.insert(uri, empty_diagnostics).is_some() {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            "\tDuplicate URIs in event payload".to_string(),
                        )
                        .await;
                }
            } else {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        "\tAttempted to delete untracked document".to_string(),
                    )
                    .await;
            }
        }

        diagnostics.append(&mut self.reload().await);
        self.send_diagnostics(diagnostics).await;
    }

    pub async fn did_delete_files(&mut self, params: DeleteFilesParams) {
        let DeleteFilesParams { files } = params;
        let mut uris = vec![];
        for FileDelete { uri } in files {
            match Url::parse(&uri) {
                Ok(uri) => uris.push(uri),
                Err(e) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("Failed to parse URI {}", e))
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
            self.send_diagnostics(diagnostics).await;
        }
    }

    pub fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
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

        if let Some(analysis) = self.analyses.get(&document.uri) {
            return Ok(Some(CompletionResponse::Array(
                analysis
                    .rule_names
                    .keys()
                    .filter(|i| partial_identifier.is_empty() || i.starts_with(partial_identifier))
                    .map(|i| CompletionItem {
                        label: i.to_owned(),
                        kind: Some(CompletionItemKind::FIELD),
                        ..Default::default()
                    })
                    .collect(),
            )));
        }

        Ok(None)
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
        let identifier = &line[range.clone()];

        if let Some(description) = get_builtin_description(identifier) {
            return Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(description.to_owned())),
                range: Some(range.into_range(text_document_position_params.position.line)),
            }));
        }

        if let Some(doc) = self
            .analyses
            .get(&document.uri)
            .and_then(|a| a.rule_docs.get(identifier))
        {
            return Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(doc.to_owned())),
                range: Some(range.into_range(text_document_position_params.position.line)),
            }));
        }

        Ok(None)
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
        let old_identifier =
            &line[line.get_word_range_at_idx(text_document_position.position.character as usize)];
        let mut edits = Vec::new();

        if let Some(references) = self
            .analyses
            .get(&document.uri)
            .and_then(|a| a.rule_occurrences.get(old_identifier))
        {
            for location in references {
                edits.push(TextEdit {
                    range: location.range,
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
                edits: edits.into_iter().map(OneOf::Left).collect(),
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

        if let Some(Some(location)) = self
            .analyses
            .get(&document.uri)
            .and_then(|a| a.rule_names.get(identifier))
        {
            return Ok(Some(GotoDeclarationResponse::Scalar(Location {
                uri: text_document_position_params.text_document.uri,
                range: location.range,
            })));
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

        if let Some(Some(location)) = self
            .analyses
            .get(&document.uri)
            .and_then(|a| a.rule_names.get(identifier))
        {
            return Ok(Some(GotoDeclarationResponse::Scalar(Location {
                uri: text_document_position_params.text_document.uri,
                range: location.range,
            })));
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

        if let Some(analysis) = self.analyses.get(&document.uri) {
            return Ok(Some(
                analysis
                    .rule_occurrences
                    .get(identifier)
                    .unwrap_or(&vec![])
                    .clone(),
            ));
        }

        Ok(None)
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

            let pairs = match parser::parse(parser::Rule::grammar_rules, document.text.as_str()) {
                Ok(pairs) => Ok(pairs),
                Err(error) => Err(vec![error]),
            };

            if let Ok(pairs) = pairs {
                if let Err(errors) = validator::validate_pairs(pairs.clone()) {
                    diagnostics.insert(
                        url.clone(),
                        PublishDiagnosticsParams::new(
                            url.clone(),
                            errors.into_diagnostics(),
                            Some(document.version),
                        ),
                    );
                } else {
                    let (_, empty_diagnostics) = create_empty_diagnostics((url, document));
                    diagnostics.insert(url.clone(), empty_diagnostics);
                }

                self.analyses
                    .entry(url.clone())
                    .or_insert_with(|| Analysis {
                        doc_url: url.clone(),
                        rule_names: BTreeMap::new(),
                        rule_occurrences: BTreeMap::new(),
                        rule_docs: BTreeMap::new(),
                    })
                    .update_from(pairs);
            } else if let Err(errors) = pairs {
                diagnostics.insert(
                    url.clone(),
                    PublishDiagnosticsParams::new(
                        url.clone(),
                        errors.into_diagnostics(),
                        Some(document.version),
                    ),
                );
            }

            if let Some(analysis) = self.analyses.get(url) {
                for (rule_name, rule_location) in
                    analysis.get_unused_rules().iter().filter(|(rule_name, _)| {
                        !self.config.always_used_rule_names.contains(rule_name)
                    })
                {
                    diagnostics
                        .entry(url.clone())
                        .or_insert_with(|| create_empty_diagnostics((url, document)).1)
                        .diagnostics
                        .push(Diagnostic::new(
                            rule_location.range,
                            Some(DiagnosticSeverity::WARNING),
                            None,
                            Some("Pest Language Server".to_owned()),
                            format!("Rule {} is unused", rule_name),
                            None,
                            None,
                        ));
                }
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
        in_dir.iter().map(create_empty_diagnostics).collect()
    }

    async fn send_diagnostics(&self, diagnostics: Diagnostics) {
        for PublishDiagnosticsParams {
            uri,
            diagnostics,
            version,
        } in diagnostics.into_values()
        {
            self.client
                .publish_diagnostics(uri.clone(), diagnostics, version)
                .await;
        }
    }
}
