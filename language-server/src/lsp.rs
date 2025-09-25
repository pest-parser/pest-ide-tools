use std::{
    collections::HashMap,
    iter,
    str::{FromStr, Split}
};

use pest_meta::parser::{self, Rule};
use serde::Deserialize;
use strum::IntoEnumIterator;
use tower_lsp::{
    Client, jsonrpc,
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
        CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
        ConfigurationItem, DeleteFilesParams, Diagnostic, DiagnosticSeverity,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
        DidOpenTextDocumentParams, DocumentChanges, DocumentFormattingParams, Documentation,
        FileChangeType, FileEvent, Hover, HoverContents, HoverParams, InitializedParams, Location,
        MarkedString, MarkupContent, MarkupKind, MessageType, OneOf,
        OptionalVersionedTextDocumentIdentifier, Position, PublishDiagnosticsParams, Range,
        ReferenceParams, RenameParams, TextDocumentEdit, TextDocumentIdentifier, TextDocumentItem,
        TextDocumentPositionParams, TextEdit, Url, VersionedTextDocumentIdentifier, WorkspaceEdit
    }
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    analysis::{Analysis, RuleAnalysis},
    builtins::Builtin,
    helpers::{
        Diagnostics, Documents, FindWordRange, IntoDiagnostics, IntoRangeWithLine, RangeContains,
        create_empty_diagnostics, str_range, validate_pairs
    }
};

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub always_used_rule_names: Vec<String>
}

#[derive(Debug)]
pub struct PestLanguageServerImpl {
    pub client: Client,
    pub documents: Documents,
    pub analyses: HashMap<Url, Analysis>,
    pub config: Config
}

impl PestLanguageServerImpl {
    async fn try_update_config(&mut self) -> Option<Config> {
        let value = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: Some("pestIdeTools".to_string())
            }])
            .await
            .ok()?
            .into_iter()
            .next()?;
        serde_json::from_value(value).ok()
    }

    pub async fn initialized(&mut self, _: InitializedParams) {
        if self.try_update_config().await.is_none() {
            self.client
                .log_message(
                    MessageType::ERROR,
                    "Failed to retrieve configuration from client"
                )
                .await;
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!("Pest Language Server v{}", env!("CARGO_PKG_VERSION"))
            )
            .await;
    }

    pub async fn shutdown(&self) -> jsonrpc::Result<()> {
        self.client
            .log_message(MessageType::INFO, "Pest Language Server shutting down :)")
            .await;
        Ok(())
    }

    pub async fn did_change_configuration(&mut self, params: DidChangeConfigurationParams) {
        let Ok(config) = serde_json::from_value(params.settings) else {
            return;
        };

        self.config = config;
        self.client
            .log_message(
                MessageType::INFO,
                "Updated configuration from client.".to_string()
            )
            .await;

        let diagnostics = self.reload().await;
        self.send_diagnostics(diagnostics).await;
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
                    "\tReopened already tracked document.".to_string()
                )
                .await;
        }

        let diagnostics = self.reload().await;
        self.send_diagnostics(diagnostics).await;
    }

    pub async fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document,
            content_changes
        } = params;
        let VersionedTextDocumentIdentifier { uri, version } = text_document;

        for change in content_changes.into_iter() {
            let updated_doc =
                TextDocumentItem::new(uri.clone(), "pest".to_owned(), version, change.text);

            if self.upsert_document(updated_doc).is_none() {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Updated untracked document {}", uri)
                    )
                    .await;
            }

            let diagnostics = self.reload().await;
            self.send_diagnostics(diagnostics).await;
        }
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
                    format!("Deleting removed document {}", uri)
                )
                .await;

            match self.remove_document(&uri) {
                Some(removed) => {
                    let empty_diagnostics = create_empty_diagnostics(uri.clone(), &removed);
                    if diagnostics.insert(uri, empty_diagnostics).is_some() {
                        self.client
                            .log_message(MessageType::WARNING, "\tDuplicate URIs in event payload")
                            .await;
                    }
                }
                None => {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            "\tAttempted to delete untracked document"
                        )
                        .await;
                }
            }
        }

        diagnostics.extend(self.reload().await);
        self.send_diagnostics(diagnostics).await;
    }

    pub async fn did_delete_files(&mut self, params: DeleteFilesParams) {
        let files = params.files;
        let mut uris = Vec::with_capacity(files.len());
        for file in files {
            match Url::parse(&file.uri) {
                Ok(uri) => uris.push(uri),
                Err(e) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("Failed to parse URI {e}"))
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
            for (uri, params) in removed {
                self.client
                    .log_message(MessageType::INFO, format!("\tDeleted {uri}"))
                    .await;

                if diagnostics.insert(uri, params).is_some() {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            "\tDuplicate URIs in event payload".to_string()
                        )
                        .await;
                }
            }
        }

        if !diagnostics.is_empty() {
            diagnostics.extend(self.reload().await);
            self.send_diagnostics(diagnostics).await;
        }
    }

    pub fn code_action(&self, params: CodeActionParams) -> Option<CodeActionResponse> {
        let CodeActionParams {
            context,
            range,
            text_document,
            ..
        } = params;
        let only = context.only.as_ref();
        let analysis = self.analyses.get(&text_document.uri)?;
        let inline_all = only
            .is_none_or(|only| only.contains(&CodeActionKind::REFACTOR_INLINE))
            .then(|| self.refactor_inline_all(analysis, &text_document, range))
            .flatten();

        let inline = only
            .is_none_or(|only| only.contains(&CodeActionKind::REFACTOR_INLINE))
            .then(|| self.refactor_inline(analysis, &text_document, range))
            .flatten();

        let extract = only
            .is_none_or(|only| only.contains(&CodeActionKind::REFACTOR_EXTRACT))
            .then(|| self.refactor_extract(range, text_document))
            .flatten();

        let actions = inline_all
            .into_iter()
            .chain(extract)
            .chain(inline)
            .map(CodeActionOrCommand::CodeAction)
            .collect();
        Some(actions)
    }

    fn refactor_inline(
        &self,
        analysis: &Analysis,
        text_document: &TextDocumentIdentifier,
        range: Range
    ) -> Option<CodeAction> {
        let ((name, ra), occurence) = analysis.rules.iter().find_map(|ra| {
            Some((
                ra,
                ra.1.occurrences
                    .iter()
                    .filter(|occurence| *occurence != &ra.1.identifier_location)
                    .find(|occurence| occurence.range.contains(range))?
            ))
        })?;

        let edit = vec![TextEdit {
            range: occurence.range,
            new_text: ra.expression.clone()
        }];

        let change = HashMap::from_iter(iter::once((text_document.uri.clone(), edit)));

        let edit = Some(WorkspaceEdit {
            changes: Some(change),
            document_changes: None,
            change_annotations: None
        });

        Some(CodeAction {
            title: format!("Inline {name}"),
            kind: Some(CodeActionKind::REFACTOR_INLINE),
            edit,
            ..Default::default()
        })
    }

    fn refactor_inline_all(
        &self,
        analysis: &Analysis,
        text_document: &TextDocumentIdentifier,
        range: Range
    ) -> Option<CodeAction> {
        let (name, ra) = analysis.rules.iter().find(|ra| {
            ra.1.occurrences
                .iter()
                .any(|occurence| occurence.range.contains(range))
        })?;

        if ra.occurrences.len() == 1 {
            return None;
        }

        let rule_expression = ra.expression.clone();

        let edits = ra
            .occurrences
            .iter()
            .filter(|occurence| ra.identifier_location.range != occurence.range)
            .map(|occurrence| TextEdit {
                range: occurrence.range,
                new_text: rule_expression.clone()
            })
            .chain(iter::once(TextEdit {
                range: ra.definition_location.range,
                new_text: String::new()
            }))
            .collect();

        let changes = HashMap::from_iter(iter::once((text_document.uri.clone(), edits)));

        let edit = Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None
        });

        Some(CodeAction {
            title: format!("Inline all occurrences of {name}"),
            kind: Some(CodeActionKind::REFACTOR_INLINE),
            edit,
            ..Default::default()
        })
    }

    fn refactor_extract(
        &self,
        range: Range,
        text_document: TextDocumentIdentifier
    ) -> Option<CodeAction> {
        let document = &self.documents[&text_document.uri];
        let line = document
            .text
            .lines()
            .nth(range.start.line as usize)
            .unwrap_or("");

        let rule_name_start_idx = line
            .graphemes(true)
            .take_while(|grapheme| *grapheme == " ")
            .count();

        let name_range = line.get_word_range_at_idx(rule_name_start_idx);
        let rule_name = &str_range(line, &name_range);

        let ra = self.get_rule_analysis(&text_document.uri, rule_name)?;
        let selected_token = ra
            .tokens
            .iter()
            .find(|(_, location)| location.range.contains(range));

        let (extracted_token, extracted_token_location) = selected_token?;
        let extracted_rule_name = (0..)
            .map(|rule_name_number| format!("{}_{}", rule_name, rule_name_number))
            .find(|extracted_rule_name| {
                self.get_rule_analysis(&text_document.uri, extracted_rule_name)
                    .is_none()
            })?;

        let extracted_rule = format!(
            "{} = {{ {} }}",
            extracted_rule_name.trim(),
            extracted_token.trim(),
        );

        let pos = Position {
            line: extracted_token_location.range.end.line + 1,
            character: 0
        };

        let prefix = if line.ends_with('\n') { "" } else { "\n" };
        let edits = vec![
            TextEdit {
                range: Range {
                    start: pos,
                    end: pos
                },
                new_text: format!("{prefix}{extracted_rule}\n",)
            },
            TextEdit {
                range: extracted_token_location.range,
                new_text: extracted_rule_name.clone()
            },
        ];

        let changes = HashMap::from_iter(iter::once((text_document.uri.clone(), edits)));

        let edit = Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None
        });

        Some(CodeAction {
            title: format!("Extract {extracted_token} into new rule ({extracted_rule_name})"),
            kind: Some(CodeActionKind::REFACTOR_EXTRACT),
            edit,
            ..Default::default()
        })
    }

    pub fn completion(&self, params: CompletionParams) -> Option<CompletionResponse> {
        let CompletionParams {
            text_document_position,
            ..
        } = params;

        let document = &self.documents[&text_document_position.text_document.uri];
        let mut lines = document.text.lines();
        let line = lines
            .nth(text_document_position.position.line as usize)
            .unwrap_or("");
        let range = line.get_word_range_at_idx(text_document_position.position.character as usize);
        let partial_identifier = &str_range(line, &range);

        let analysis = self.analyses.get(&document.uri)?;
        let rule_completions = analysis
            .rules
            .iter()
            .filter(|(i, _)| i.starts_with(partial_identifier))
            .map(|(i, ra)| CompletionItem {
                label: i.to_owned(),
                kind: Some(CompletionItemKind::FIELD),
                documentation: ra
                    .doc
                    .clone()
                    .map(|value| MarkupContent {
                        kind: MarkupKind::Markdown,
                        value
                    })
                    .map(Documentation::MarkupContent),
                ..Default::default()
            });

        let builtins_completions = Builtin::iter().map(|builtin| CompletionItem {
            label: builtin.as_ref().to_string(),
            kind: Some(builtin.kind()),
            documentation: Some(Documentation::String(builtin.description().to_string())),
            ..Default::default()
        });

        let completions = rule_completions.chain(builtins_completions).collect();
        Some(CompletionResponse::Array(completions))
    }

    pub fn hover(&self, params: HoverParams) -> Option<Hover> {
        let HoverParams {
            text_document_position_params,
            ..
        } = params;
        let document = &self.documents[&text_document_position_params.text_document.uri];

        let mut lines = document.text.lines();
        let line = lines
            .nth(text_document_position_params.position.line as usize)
            .unwrap_or("");
        let range =
            line.get_word_range_at_idx(text_document_position_params.position.character as usize);
        let identifier = &str_range(line, &range);

        if let Ok(builtin) = Builtin::from_str(identifier) {
            let contents =
                HoverContents::Scalar(MarkedString::String(builtin.description().to_owned()));
            let range = Some(range.into_range(text_document_position_params.position.line));
            let hover = Hover { contents, range };
            return Some(hover);
        }

        let ra = self.analyses.get(&document.uri)?.rules.get(identifier)?;

        let contents = HoverContents::Scalar(MarkedString::String(ra.doc.clone()?));
        let range = Some(range.into_range(text_document_position_params.position.line));
        Some(Hover { contents, range })
    }

    pub fn rename(&self, params: RenameParams) -> WorkspaceEdit {
        let RenameParams {
            text_document_position,
            new_name,
            ..
        } = params;

        let document = &self.documents[&text_document_position.text_document.uri];
        let line = document
            .text
            .lines()
            .nth(text_document_position.position.line as usize)
            .unwrap_or("");
        let old_identifier = &str_range(
            line,
            &line.get_word_range_at_idx(text_document_position.position.character as usize)
        );

        let edits = self
            .get_rule_analysis(&document.uri, old_identifier)
            .into_iter()
            .flat_map(|ra| &ra.occurrences)
            .map(|location| TextEdit {
                range: location.range,
                new_text: new_name.clone()
            })
            .map(OneOf::Left)
            .collect();

        let text_document = OptionalVersionedTextDocumentIdentifier {
            uri: text_document_position.text_document.uri,
            version: Some(document.version)
        };

        let edit = TextDocumentEdit {
            text_document,
            edits
        };

        let document_changes = Some(DocumentChanges::Edits(vec![edit]));

        WorkspaceEdit {
            change_annotations: None,
            changes: None,
            document_changes
        }
    }

    pub fn goto_definition(&self, params: TextDocumentPositionParams) -> Option<Location> {
        let uri = params.text_document.uri;
        let document = &self.documents[&uri];
        let mut lines = document.text.lines();
        let line = lines.nth(params.position.line as usize).unwrap_or("");
        let range = line.get_word_range_at_idx(params.position.character as usize);
        let identifier = &str_range(line, &range);

        let Location { range, .. } = self
            .get_rule_analysis(&document.uri, identifier)?
            .definition_location;
        Some(Location { uri, range })
    }

    pub fn references(&self, params: ReferenceParams) -> Option<Vec<Location>> {
        let ReferenceParams {
            text_document_position,
            ..
        } = params;

        let document = &self.documents[&text_document_position.text_document.uri];

        let mut lines = document.text.lines();
        let line = lines
            .nth(text_document_position.position.line as usize)
            .unwrap_or("");
        let range = line.get_word_range_at_idx(text_document_position.position.character as usize);
        let identifier = &str_range(line, &range);

        let rule_analysis = self.get_rule_analysis(&document.uri, identifier)?;
        let locations = rule_analysis.occurrences.clone();
        Some(locations)
    }

    pub fn formatting(&self, params: DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let DocumentFormattingParams { text_document, .. } = params;

        let document = &self.documents[&text_document.uri];
        let input = document.text.as_str();

        let fmt = pest_fmt::Formatter::new(input);
        let formatted = fmt.format().ok()?;
        let lines = document.text.lines();
        let last_line = lines.clone().last().unwrap_or("");
        let end = Position::new(lines.count() as u32, last_line.len() as u32);
        let range = Range::new(Position::new(0, 0), end);
        Some(vec![TextEdit::new(range, formatted)])
    }
}

impl PestLanguageServerImpl {
    async fn analyse_document(
        analyses: &mut HashMap<Url, Analysis>,
        diagnostics: &mut Diagnostics,
        config: &Config,
        url: Url,
        document: &TextDocumentItem
    ) -> Result<(), Vec<pest::error::Error<Rule>>> {
        let pairs = parser::parse(parser::Rule::grammar_rules, document.text.as_str())
            .map_err(|err| vec![err])?;
        let analysis = analyses.entry(url.clone()).or_insert(Analysis {
            doc_url: url.clone(),
            rules: HashMap::new()
        });
        analysis.update_from(pairs.clone());

        let unused_diagnostics: Vec<_> = analysis
            .get_unused_rules()
            .iter()
            .filter(|(rule_name, _)| !config.always_used_rule_names.contains(rule_name))
            .map(|(rule_name, rule_location)| Diagnostic {
                range: rule_location.range,
                severity: Some(DiagnosticSeverity::WARNING),
                source: Some("Pest Language Server".to_owned()),
                message: format!("Rule {} is unused", rule_name),
                ..Default::default()
            })
            .collect();

        if unused_diagnostics.len() > 1 {
            diagnostics
                .entry(url.clone())
                .or_insert_with(|| create_empty_diagnostics(url, document))
                .diagnostics
                .extend(unused_diagnostics);
        }

        validate_pairs(pairs)
    }

    async fn reload(&mut self) -> Diagnostics {
        self.client
            .log_message(MessageType::INFO, "Reloading all diagnostics".to_string())
            .await;
        let mut diagnostics = Diagnostics::new();

        for (url, document) in &self.documents {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("\tReloading diagnostics for {}", url)
                )
                .await;

            if let Err(error) = Self::analyse_document(
                &mut self.analyses,
                &mut diagnostics,
                &self.config,
                url.clone(),
                document
            )
            .await
            {
                let v = PublishDiagnosticsParams::new(
                    url.clone(),
                    error.into_diagnostics(),
                    Some(document.version)
                );

                diagnostics.insert(url.clone(), v)
            } else {
                let empty_diagnostics = create_empty_diagnostics(url.clone(), document);
                diagnostics.insert(url.clone(), empty_diagnostics)
            };
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
                maybe_segments.is_some_and(compare_paths)
            });

        self.documents = not_in_dir;
        in_dir
            .into_iter()
            .map(|(url, doc)| (url.clone(), create_empty_diagnostics(url, &doc)))
            .collect()
    }

    async fn send_diagnostics(&self, diagnostics: Diagnostics) {
        for PublishDiagnosticsParams {
            uri,
            diagnostics,
            version
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
    }
}
