use std::{collections::HashMap, str::Split};

use pest_meta::parser;
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        request::{GotoDeclarationParams, GotoDeclarationResponse},
        CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
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
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    analysis::{Analysis, RuleAnalysis},
    builtins::BUILTINS,
    config::Config,
    helpers::{
        create_empty_diagnostics, range_contains, str_range, validate_pairs, Diagnostics,
        Documents, FindWordRange, IntoDiagnostics, IntoRangeWithLine,
    },
};
use crate::{builtins::get_builtin_description, update_checker::check_for_updates};

#[derive(Debug)]
pub struct PestLanguageServerImpl {
    pub client: Client,
    pub documents: Documents,
    pub analyses: HashMap<Url, Analysis>,
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
            self.client
                .log_message(MessageType::INFO, "Checking for updates...".to_string())
                .await;

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
        if let Ok(config) = serde_json::from_value(params.settings) {
            self.config = config;
            self.client
                .log_message(
                    MessageType::INFO,
                    "Updated configuration from client.".to_string(),
                )
                .await;

            let diagnostics = self.reload().await;
            self.send_diagnostics(diagnostics).await;
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

        diagnostics.extend(self.reload().await);
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
            diagnostics.extend(self.reload().await);
            self.send_diagnostics(diagnostics).await;
        }
    }

    pub fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let CodeActionParams {
            context,
            range,
            text_document,
            ..
        } = params;
        let only = context.only;
        let analysis = self.analyses.get(&text_document.uri);
        let mut actions = Vec::new();

        if let Some(analysis) = analysis {
            // Inlining
            if only
                .as_ref()
                .map_or(true, |only| only.contains(&CodeActionKind::REFACTOR_INLINE))
            {
                let mut rule_name = None;

                for (name, ra) in analysis.rules.iter() {
                    if let Some(ra) = ra {
                        if ra.identifier_location.range == range {
                            rule_name = Some(name);
                            break;
                        }
                    }
                }

                if let Some(rule_name) = rule_name {
                    let ra = self
                        .get_rule_analysis(&text_document.uri, rule_name)
                        .expect("should not be called on a builtin with no rule analysis");
                    let rule_expression = ra.expression.clone();

                    let mut edits = Vec::new();

                    edits.push(TextEdit {
                        range: ra.definition_location.range,
                        new_text: "".to_owned(),
                    });

                    if let Some(occurrences) = self
                        .get_rule_analysis(&text_document.uri, rule_name)
                        .map(|ra| &ra.occurrences)
                    {
                        for occurrence in occurrences {
                            if occurrence.range != ra.identifier_location.range {
                                edits.push(TextEdit {
                                    range: occurrence.range,
                                    new_text: rule_expression.clone(),
                                });
                            }
                        }
                    }

                    let mut changes = HashMap::new();
                    changes.insert(text_document.uri.clone(), edits);

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Inline rule".to_owned(),
                        kind: Some(CodeActionKind::REFACTOR_INLINE),
                        edit: Some(WorkspaceEdit {
                            changes: Some(changes),
                            document_changes: None,
                            change_annotations: None,
                        }),
                        ..Default::default()
                    }));
                }
            }

            if only.as_ref().map_or(true, |only| {
                only.contains(&CodeActionKind::REFACTOR_EXTRACT)
            }) && range.start.line == range.end.line
            {
                let document = self.documents.get(&text_document.uri).unwrap();
                let mut lines = document.text.lines();
                let line = lines.nth(range.start.line as usize).unwrap_or("");

                let mut rule_name_start_idx = 0;
                let mut chars = line.graphemes(true);

                while chars.next() == Some(" ") {
                    rule_name_start_idx += 1;
                }

                let name_range = line.get_word_range_at_idx(rule_name_start_idx);
                let rule_name = &str_range(line, &name_range);

                if let Some(ra) = self.get_rule_analysis(&text_document.uri, rule_name) {
                    let mut selected_token = None;

                    for (token, location) in ra.tokens.iter() {
                        if range_contains(&location.range, &range) {
                            selected_token = Some((token, location));
                            break;
                        }
                    }

                    if let Some((extracted_token, location)) = selected_token {
                        //TODO: Replace with something more robust, it's horrible
                        let extracted_token_identifier =
                            match parser::parse(parser::Rule::node, extracted_token) {
                                Ok(mut node) => {
                                    let mut next = node.next().unwrap();

                                    loop {
                                        match next.as_rule() {
                                            parser::Rule::terminal => {
                                                next = next.into_inner().next().unwrap();
                                            }
                                            parser::Rule::_push
                                            | parser::Rule::peek_slice
                                            | parser::Rule::identifier
                                            | parser::Rule::string
                                            | parser::Rule::insensitive_string
                                            | parser::Rule::range => {
                                                break Some(next);
                                            }
                                            parser::Rule::opening_paren => {
                                                node = node.next().unwrap().into_inner();
                                                let next_opt = node
                                                    .find(|r| r.as_rule() == parser::Rule::term)
                                                    .unwrap()
                                                    .into_inner()
                                                    .find(|r| r.as_rule() == parser::Rule::node);

                                                if let Some(new_next) = next_opt {
                                                    next = new_next;
                                                } else {
                                                    break None;
                                                }
                                            }
                                            _ => unreachable!(
                                                "unexpected rule in node: {:?}",
                                                next.as_rule()
                                            ),
                                        };
                                    }
                                    .map(|p| p.as_str())
                                }
                                Err(_) => None,
                            }
                            .unwrap_or("");

                        if self
                            .get_rule_analysis(&text_document.uri, extracted_token_identifier)
                            .is_some()
                            || BUILTINS.contains(&extracted_token_identifier)
                            || extracted_token.starts_with('\"')
                            || extracted_token.starts_with('\'')
                            || extracted_token.starts_with("PUSH")
                            || extracted_token.starts_with("PEEK")
                            || extracted_token.starts_with("^\"")
                        {
                            let mut rule_name_number = 0;
                            let extracted_rule_name = loop {
                                rule_name_number += 1;
                                let extracted_rule_name =
                                    format!("{}_{}", rule_name, rule_name_number);
                                if self
                                    .get_rule_analysis(&text_document.uri, &extracted_rule_name)
                                    .is_none()
                                {
                                    break extracted_rule_name;
                                }
                            };

                            let extracted_rule = format!(
                                "{} = {{ {} }}",
                                extracted_rule_name.trim(),
                                extracted_token.trim(),
                            );

                            let mut edits = Vec::new();

                            edits.push(TextEdit {
                                range: Range {
                                    start: Position {
                                        line: location.range.end.line + 1,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: location.range.end.line + 1,
                                        character: 0,
                                    },
                                },
                                new_text: format!(
                                    "{}{}\n",
                                    if line.ends_with('\n') { "" } else { "\n" },
                                    extracted_rule
                                ),
                            });

                            let mut changes = HashMap::new();
                            changes.insert(text_document.uri.clone(), edits);

                            for (url, analysis) in self.analyses.iter() {
                                for (_, ra) in analysis.rules.iter() {
                                    if let Some(ra) = ra {
                                        for (token, location) in ra.tokens.iter() {
                                            if token == extracted_token {
                                                changes
                                                    .entry(url.clone())
                                                    .or_insert_with(Vec::new)
                                                    .push(TextEdit {
                                                        range: location.range,
                                                        new_text: format!(
                                                            "{} ",
                                                            extracted_rule_name
                                                        ),
                                                    });
                                            }
                                        }
                                    }
                                }
                            }

                            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: "Extract into new rule".to_owned(),
                                kind: Some(CodeActionKind::REFACTOR_EXTRACT),
                                edit: Some(WorkspaceEdit {
                                    changes: Some(changes),
                                    document_changes: None,
                                    change_annotations: None,
                                }),
                                ..Default::default()
                            }));
                        }
                    }
                }
            }
        }

        Ok(Some(actions))
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
        let partial_identifier = &str_range(line, &range);

        if let Some(analysis) = self.analyses.get(&document.uri) {
            return Ok(Some(CompletionResponse::Array(
                analysis
                    .rules
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
        let identifier = &str_range(line, &range);

        if let Some(description) = get_builtin_description(identifier) {
            return Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(description.to_owned())),
                range: Some(range.into_range(text_document_position_params.position.line)),
            }));
        }

        if let Some(Some(ra)) = self
            .analyses
            .get(&document.uri)
            .and_then(|a| a.rules.get(identifier))
        {
            return Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(ra.doc.clone())),
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
        let old_identifier = &str_range(
            line,
            &line.get_word_range_at_idx(text_document_position.position.character as usize),
        );
        let mut edits = Vec::new();

        if let Some(occurrences) = self
            .get_rule_analysis(&document.uri, old_identifier)
            .map(|ra| &ra.occurrences)
        {
            for location in occurrences {
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
        let identifier = &str_range(line, &range);

        if let Some(location) = self
            .get_rule_analysis(&document.uri, identifier)
            .map(|ra| &ra.definition_location)
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
        let identifier = &str_range(line, &range);

        if let Some(location) = self
            .get_rule_analysis(&document.uri, identifier)
            .map(|ra| &ra.definition_location)
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
        let identifier = &str_range(line, &range);

        Ok(self
            .get_rule_analysis(&document.uri, identifier)
            .map(|ra| ra.occurrences.clone()))
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
                if let Err(errors) = validate_pairs(pairs.clone()) {
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
                        rules: HashMap::new(),
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
                let mut unused_diagnostics = Vec::new();
                for (rule_name, rule_location) in
                    analysis.get_unused_rules().iter().filter(|(rule_name, _)| {
                        !self.config.always_used_rule_names.contains(rule_name)
                    })
                {
                    unused_diagnostics.push(Diagnostic::new(
                        rule_location.range,
                        Some(DiagnosticSeverity::WARNING),
                        None,
                        Some("Pest Language Server".to_owned()),
                        format!("Rule {} is unused", rule_name),
                        None,
                        None,
                    ));
                }

                if unused_diagnostics.len() > 1 {
                    diagnostics
                        .entry(url.to_owned())
                        .or_insert_with(|| create_empty_diagnostics((url, document)).1)
                        .diagnostics
                        .extend(unused_diagnostics);
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

    fn get_rule_analysis(&self, uri: &Url, rule_name: &str) -> Option<&RuleAnalysis> {
        self.analyses
            .get(uri)
            .and_then(|analysis| analysis.rules.get(rule_name))
            .and_then(|ra| ra.as_ref())
    }
}
