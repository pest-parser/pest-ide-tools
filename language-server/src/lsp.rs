use std::{collections::HashMap, iter, str::FromStr};

use pest::error::Error;
use pest_meta::parser::{self, Rule};
use serde::Deserialize;
use strum::IntoEnumIterator;
use tower_lsp::{
    Client, jsonrpc,
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
        CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
        ConfigurationItem, DeleteFilesParams, Diagnostic, DiagnosticSeverity,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
        DocumentChanges, DocumentFormattingParams, DocumentSymbolParams, DocumentSymbolResponse,
        Documentation, Hover, HoverContents, HoverParams, InitializedParams, Location,
        MarkedString, MarkupContent, MarkupKind, MessageType, OneOf,
        OptionalVersionedTextDocumentIdentifier, Position, PublishDiagnosticsParams, Range,
        ReferenceParams, RenameParams, SymbolInformation, SymbolKind, TextDocumentEdit,
        TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, TextEdit, Url,
        VersionedTextDocumentIdentifier, WorkspaceEdit,
    },
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    analysis::{Analysis, RuleAnalysis},
    builtins::Builtin,
    helpers::{
        Diagnostics, Documents, FindWordRange, IntoDiagnostics, IntoRangeWithLine, RangeContains,
        str_range, validate_pairs,
    },
};

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub always_used_rule_names: Vec<String>,
}

#[derive(Debug)]
pub struct PestLanguageServerImpl {
    client: Client,
    documents: Documents,
    analyses: HashMap<Url, Analysis>,
    config: Config,
}

impl PestLanguageServerImpl {
    pub fn new(client: Client) -> Self {
        Self {
            analyses: HashMap::new(),
            client,
            config: Config::default(),
            documents: HashMap::new(),
        }
    }

    async fn try_update_config(&mut self) -> Option<Config> {
        let value = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: Some("pestIdeTools".to_string()),
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
                    "Failed to retrieve configuration from client",
                )
                .await;
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!("Pest Language Server v{}", env!("CARGO_PKG_VERSION")),
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
                "Updated configuration from client.".to_string(),
            )
            .await;

        let diagnostics = self.reload().await;
        self.send_diagnostics(diagnostics).await;
    }

    pub async fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let DidOpenTextDocumentParams { text_document } = params;
        if self
            .documents
            .insert(text_document.uri.clone(), text_document)
            .is_some()
        {
            self.client
                .log_message(
                    MessageType::ERROR,
                    "Reopened already tracked document.".to_string(),
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

        let Some(change) = content_changes.into_iter().next_back() else {
            self.client
                .log_message(MessageType::ERROR, "Editor returned empty change vector")
                .await;
            return;
        };

        let updated_doc =
            TextDocumentItem::new(uri.clone(), "pest".to_owned(), version, change.text);
        let Some(document) = self.documents.get_mut(&uri) else {
            self.client
                .log_message(MessageType::ERROR, "Editor changed nonexistent document")
                .await;
            return;
        };

        *document = updated_doc;
        let diagnostics = self.reload().await;
        self.send_diagnostics(diagnostics).await;
    }

    pub async fn did_delete_files(&mut self, params: DeleteFilesParams) {
        let files = params.files;
        for file in files {
            match Url::parse(&file.uri) {
                Ok(uri) => _ = self.documents.remove(&uri),
                Err(e) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("Failed to parse URI {e}"))
                        .await
                }
            }
        }

        let diagnostics = self.reload().await;
        self.send_diagnostics(diagnostics).await;
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
        range: Range,
    ) -> Option<CodeAction> {
        let ((name, ra), reference) = analysis.rules.iter().find_map(|pair @ (_, ra)| {
            Some((
                pair,
                ra.references
                    .iter()
                    .find(|reference| reference.contains(range))?,
            ))
        })?;

        let edit = vec![TextEdit {
            range: *reference,
            new_text: ra.expression.clone(),
        }];

        let change = HashMap::from_iter(iter::once((text_document.uri.clone(), edit)));

        let edit = Some(WorkspaceEdit {
            changes: Some(change),
            document_changes: None,
            change_annotations: None,
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
        range: Range,
    ) -> Option<CodeAction> {
        let (name, ra) = analysis.rules.iter().find(|(_, ra)| {
            (ra.identifier_location.contains(range) && !ra.references.is_empty())
                || ra
                    .references
                    .iter()
                    .any(|reference| reference.contains(range))
        })?;

        let rule_expression = ra.expression.clone();

        let edits = ra
            .references
            .iter()
            .map(|reference| TextEdit {
                range: *reference,
                new_text: rule_expression.clone(),
            })
            .chain(iter::once(TextEdit {
                range: ra.definition_location,
                new_text: String::new(),
            }))
            .collect();

        let changes = HashMap::from_iter(iter::once((text_document.uri.clone(), edits)));

        let edit = Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
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
        text_document: TextDocumentIdentifier,
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

        let name_range = line.word_range_at_idx(rule_name_start_idx);
        let rule_name = &str_range(line, &name_range);

        let ra = self.rule_analysis(&text_document.uri, rule_name)?;
        let selected_token = ra
            .tokens
            .iter()
            .find(|(_, location)| location.contains(range));

        let (extracted_token, extracted_token_location) = selected_token?;
        let extracted_rule_name = (0..)
            .map(|rule_name_number| format!("{}_{}", rule_name, rule_name_number))
            .find(|extracted_rule_name| {
                self.rule_analysis(&text_document.uri, extracted_rule_name)
                    .is_none()
            })?;

        let extracted_rule = format!(
            "{} = {{ {} }}",
            extracted_rule_name.trim(),
            extracted_token.trim(),
        );

        let pos = Position {
            line: extracted_token_location.end.line + 1,
            character: 0,
        };

        let prefix = if line.ends_with('\n') { "" } else { "\n" };
        let edits = vec![
            TextEdit {
                range: Range {
                    start: pos,
                    end: pos,
                },
                new_text: format!("{prefix}{extracted_rule}\n",),
            },
            TextEdit {
                range: *extracted_token_location,
                new_text: extracted_rule_name.clone(),
            },
        ];

        let changes = HashMap::from_iter(iter::once((text_document.uri.clone(), edits)));

        let edit = Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        });

        Some(CodeAction {
            title: format!("Extract {extracted_token} into {extracted_rule_name}"),
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
        let range = line.word_range_at_idx(text_document_position.position.character as usize);
        let partial_identifier = &str_range(line, &range);

        let analysis = self.analyses.get(&document.uri)?;
        let rule_completions = analysis
            .rules
            .iter()
            .filter(|(name, _)| name.starts_with(partial_identifier))
            .map(|(name, ra)| CompletionItem {
                label: name.to_owned(),
                kind: Some(CompletionItemKind::FIELD),
                documentation: ra
                    .doc
                    .clone()
                    .map(|value| MarkupContent {
                        kind: MarkupKind::Markdown,
                        value,
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
            line.word_range_at_idx(text_document_position_params.position.character as usize);
        let identifier = &str_range(line, &range);

        if let Ok(builtin) = Builtin::from_str(identifier) {
            let contents =
                HoverContents::Scalar(MarkedString::String(builtin.description().to_owned()));
            let range = Some(range.into_range(text_document_position_params.position.line));
            let hover = Hover { contents, range };
            return Some(hover);
        }

        let ra = self
            .analyses
            .get(&document.uri)?
            .rules
            .iter()
            .find(|(name, _)| *name == identifier)?
            .1;

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
            &line.word_range_at_idx(text_document_position.position.character as usize),
        );

        let edits = self
            .rule_analysis(&document.uri, old_identifier)
            .into_iter()
            .flat_map(|ra| ra.references_and_identifier())
            .map(|range| TextEdit {
                range,
                new_text: new_name.clone(),
            })
            .map(OneOf::Left)
            .collect();

        let text_document = OptionalVersionedTextDocumentIdentifier {
            uri: text_document_position.text_document.uri,
            version: Some(document.version),
        };

        let edit = TextDocumentEdit {
            text_document,
            edits,
        };

        let document_changes = Some(DocumentChanges::Edits(vec![edit]));

        WorkspaceEdit {
            change_annotations: None,
            changes: None,
            document_changes,
        }
    }

    pub fn goto_definition(&self, params: TextDocumentPositionParams) -> Option<Location> {
        let uri = params.text_document.uri;
        let document = &self.documents[&uri];
        let mut lines = document.text.lines();
        let line = lines.nth(params.position.line as usize).unwrap_or("");
        let range = line.word_range_at_idx(params.position.character as usize);
        let identifier = &str_range(line, &range);

        let range = self
            .rule_analysis(&document.uri, identifier)?
            .definition_location;
        Some(Location { uri, range })
    }

    pub fn references(&self, params: ReferenceParams) -> Option<Vec<Location>> {
        let ReferenceParams {
            text_document_position,
            ..
        } = params;

        let uri = text_document_position.text_document.uri;
        let document = &self.documents[&uri];

        let mut lines = document.text.lines();
        let line = lines
            .nth(text_document_position.position.line as usize)
            .unwrap_or("");
        let range = line.word_range_at_idx(text_document_position.position.character as usize);
        let identifier = &str_range(line, &range);

        let rule_analysis = self.rule_analysis(&document.uri, identifier)?;
        let locations = rule_analysis
            .references_and_identifier()
            .map(|range| Location {
                uri: uri.clone(),
                range,
            })
            .collect();
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

    #[allow(deprecated)]
    pub fn document_symbol(&self, params: DocumentSymbolParams) -> Option<DocumentSymbolResponse> {
        let uri = params.text_document.uri;
        let analysis = self.analyses.get(&uri)?;
        Some(DocumentSymbolResponse::Flat(
            analysis
                .rules
                .iter()
                .map(|(name, ra)| SymbolInformation {
                    name: name.to_owned(),
                    kind: SymbolKind::FIELD,
                    tags: None,
                    // Stupid library forces me to specify a deprecated field like what
                    deprecated: None,
                    location: Location {
                        uri: uri.clone(),
                        range: ra.identifier_location,
                    },
                    container_name: None,
                })
                .collect(),
        ))
    }

    fn analyse_document(
        analyses: &mut HashMap<Url, Analysis>,
        config: &Config,
        uri: Url,
        document: &TextDocumentItem,
    ) -> Result<Vec<Diagnostic>, Vec<Error<Rule>>> {
        let pairs =
            parser::parse(Rule::grammar_rules, document.text.as_str()).map_err(|err| vec![err])?;
        let analysis = analyses.entry(uri.clone()).or_insert(Analysis {
            rules: HashMap::new(),
        });

        analysis.update_from(pairs.clone());

        let unused_rules = analysis.unused_rules();
        let mut unused_diagnostics: Vec<_> = unused_rules
            .filter(|(rule_name, _)| {
                !config
                    .always_used_rule_names
                    .iter()
                    .any(|name| name == rule_name)
            })
            .map(|(rule_name, range)| Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::WARNING),
                source: Some("Pest Language Server".to_owned()),
                message: format!("Rule {} is unused", rule_name),
                ..Default::default()
            })
            .collect();

        if config.always_used_rule_names.is_empty() && unused_diagnostics.len() == 1 {
            unused_diagnostics.clear();
        }

        validate_pairs(pairs).map(|_| unused_diagnostics)
    }

    async fn reload(&mut self) -> Diagnostics {
        self.client
            .log_message(MessageType::INFO, "Reloading all diagnostics".to_string())
            .await;
        self.documents
            .iter()
            .map(|(url, document)| {
                let diagnostics =
                    Self::analyse_document(&mut self.analyses, &self.config, url.clone(), document)
                        .unwrap_or_else(|errors| errors.into_diagnostics());

                PublishDiagnosticsParams::new(url.clone(), diagnostics, Some(document.version))
            })
            .collect()
    }

    async fn send_diagnostics(&self, diagnostics: Diagnostics) {
        for PublishDiagnosticsParams {
            uri,
            diagnostics,
            version,
        } in diagnostics
        {
            self.client
                .publish_diagnostics(uri.clone(), diagnostics, version)
                .await;
        }
    }

    fn rule_analysis(&self, uri: &Url, rule_name: &str) -> Option<&RuleAnalysis> {
        self.analyses
            .get(uri)
            .and_then(|analysis| analysis.rules.get(rule_name))
    }
}
