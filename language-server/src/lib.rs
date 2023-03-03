use crate::helpers::{IntoLocation, IntoRangeWithLine};
use crate::{builtins::BUILTINS, helpers::FindAllOccurrences};
use std::{collections::BTreeMap, str::Split};

use lsp_types::{
    notification::{
        DidChangeTextDocument, DidChangeWatchedFiles, DidDeleteFiles, DidOpenTextDocument,
        Notification,
    },
    request::{GotoDeclarationParams, GotoDeclarationResponse},
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, DeleteFilesParams,
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidOpenTextDocumentParams, DocumentChanges, FileChangeType, FileDelete, FileEvent,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams, Location,
    MarkedString, OneOf, OptionalVersionedTextDocumentIdentifier, PublishDiagnosticsParams, Range,
    ReferenceParams, RenameParams, TextDocumentEdit, TextDocumentItem, TextEdit, Url,
    VersionedTextDocumentIdentifier, WorkspaceEdit,
};
use lsp_types::{DocumentFormattingParams, Position};
use pest::error::ErrorVariant;
use pest_meta::parser::Rule;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

mod builtins;
mod helpers;

use builtins::get_builtin_description;
use helpers::{get_empty_diagnostics, log, parse_pest_grammar, Diagnostics, Documents};

use crate::helpers::{FindWord, IntoRange};

#[wasm_bindgen]
pub struct PestLanguageServer {
    documents: Documents,
    diagnostics_callback: js_sys::Function,
}

#[wasm_bindgen]
impl PestLanguageServer {
    #[wasm_bindgen(constructor)]
    pub fn new(send_diagnostics_callback: &js_sys::Function) -> Self {
        console_error_panic_hook::set_once();

        Self {
            documents: BTreeMap::new(),
            diagnostics_callback: send_diagnostics_callback.clone(),
        }
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = PestLanguageServer, js_name = onFileNotification)]
    pub fn on_file_notification(&mut self, method: &str, params: JsValue) {
        match method {
            DidOpenTextDocument::METHOD => {
                let DidOpenTextDocumentParams { text_document } = from_value(params).unwrap();
                log(&format!("Opening {}", text_document.uri));

                if self.upsert_document(text_document).is_some() {
                    log("\tReopened already tracked document.");
                }

                let diagnostics = self.reload();
                self.send_diagnostics(&diagnostics);
            }

            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = from_value(params).unwrap();

                assert_eq!(params.content_changes.len(), 1);
                let change = params.content_changes.into_iter().next().unwrap();
                assert!(change.range.is_none());

                let VersionedTextDocumentIdentifier { uri, version } = params.text_document;

                let updated_doc =
                    TextDocumentItem::new(uri.clone(), "pest".to_owned(), version, change.text);

                if self.upsert_document(updated_doc).is_none() {
                    log(&format!("Updated untracked document {}", uri));
                }

                let diagnostics = self.reload();
                self.send_diagnostics(&diagnostics);
            }

            DidChangeWatchedFiles::METHOD => {
                let DidChangeWatchedFilesParams { changes } = from_value(params).unwrap();
                let uris: Vec<_> = changes
                    .into_iter()
                    .map(|FileEvent { uri, typ }| {
                        assert_eq!(typ, FileChangeType::DELETED);
                        uri
                    })
                    .collect();

                let mut diagnostics = Diagnostics::new();

                for uri in uris {
                    log(&format!("Deleting removed document {}", uri));

                    if let Some(removed) = self.remove_document(&uri) {
                        let (_, empty_diagnostics) = get_empty_diagnostics((&uri, &removed));
                        if diagnostics.insert(uri, empty_diagnostics).is_some() {
                            log("\tDuplicate URIs in event payload");
                        }
                    } else {
                        log("\tAttempted to delete untracked document");
                    }
                }

                diagnostics.append(&mut self.reload());
                self.send_diagnostics(&diagnostics);
            }

            DidDeleteFiles::METHOD => {
                let DeleteFilesParams { files } = from_value(params).unwrap();
                let mut uris = vec![];
                for FileDelete { uri } in files {
                    match Url::parse(&uri) {
                        Ok(uri) => uris.push(uri),
                        Err(e) => log(&format!("Failed to parse URI {}", e)),
                    }
                }

                let mut diagnostics = Diagnostics::new();

                log(&format!("Deleting {} files", uris.len()));

                for uri in uris {
                    let removed = self.remove_documents_in_dir(&uri);
                    if !removed.is_empty() {
                        for (uri, params) in removed {
                            log(&format!("\tDeleted {}", uri));

                            if diagnostics.insert(uri, params).is_some() {
                                log("\tDuplicate URIs in event payload");
                            }
                        }
                    }
                }

                if !diagnostics.is_empty() {
                    diagnostics.append(&mut self.reload());
                    self.send_diagnostics(&diagnostics);
                }
            }

            _ => (),
        }
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = PestLanguageServer, js_name = onRename)]
    pub fn on_rename(&mut self, params: JsValue) -> JsValue {
        let RenameParams {
            text_document_position,
            new_name,
            ..
        } = from_value(params).unwrap();

        let document = self
            .documents
            .get(&text_document_position.text_document.uri)
            .unwrap();

        let old_name = &document.text[document
            .text
            .lines()
            .nth(text_document_position.position.line as usize)
            .unwrap_or("")
            .get_word_range_at_idx(text_document_position.position.character as usize)];

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

        to_value(&WorkspaceEdit {
            change_annotations: None,
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document_position.text_document.uri,
                    version: Some(document.version),
                },
                edits: edits.iter().map(|edit| OneOf::Left(edit.clone())).collect(),
            }])),
        })
        .unwrap()
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = PestLanguageServer, js_name = onHover)]
    pub fn on_hover(&mut self, params: JsValue) -> JsValue {
        let HoverParams {
            text_document_position_params,
            ..
        } = from_value(params).unwrap();

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

        if let Some(description) = get_builtin_description(word) {
            to_value(&Hover {
                contents: HoverContents::Scalar(MarkedString::String(description.to_owned())),
                range: Some(range.into_lsp_range(text_document_position_params.position.line)),
            })
            .unwrap()
        } else {
            to_value(&Hover {
                contents: HoverContents::Scalar(MarkedString::String("".to_string())),
                range: Some(range.into_lsp_range(text_document_position_params.position.line)),
            })
            .unwrap()
        }
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = PestLanguageServer, js_name = onGotoDeclaration)]
    pub fn on_goto_declaration(&mut self, params: JsValue) -> JsValue {
        let GotoDeclarationParams {
            text_document_position_params,
            ..
        } = from_value(params).unwrap();

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
            for pair in pairs.into_iter() {
                if pair.as_rule() == Rule::grammar_rule {
                    let mut inner = pair.into_inner();
                    let inner = inner.next().unwrap();

                    if inner.as_str() == identifier {
                        definition = Some(inner.as_span().into_lsp_range());
                    }
                }

                if let Some(definition) = definition {
                    return to_value(&GotoDeclarationResponse::Scalar(Location {
                        uri: text_document_position_params.text_document.uri,
                        range: definition,
                    }))
                    .unwrap();
                }
            }
        }

        JsValue::null()
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = PestLanguageServer, js_name = onGotoDefinition)]
    pub fn on_goto_definition(&mut self, params: JsValue) -> JsValue {
        let GotoDefinitionParams {
            text_document_position_params,
            ..
        } = from_value(params).unwrap();

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
            for pair in pairs.into_iter() {
                if pair.as_rule() == Rule::grammar_rule {
                    let mut inner = pair.into_inner();
                    let inner = inner.next().unwrap();

                    if inner.as_str() == identifier {
                        definition = Some(inner.as_span().into_lsp_range());
                    }
                }

                if let Some(definition) = definition {
                    return to_value(&GotoDefinitionResponse::Scalar(Location {
                        uri: text_document_position_params.text_document.uri,
                        range: definition,
                    }))
                    .unwrap();
                }
            }
        }

        JsValue::null()
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = PestLanguageServer, js_name = onFindReferences)]
    pub fn on_find_references(&mut self, params: JsValue) -> JsValue {
        let ReferenceParams {
            text_document_position,
            ..
        } = from_value(params).unwrap();

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

        to_value(&references).unwrap()
    }

    //TODO(Jamalam360): Only suggest rule names in relevant situations.
    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = PestLanguageServer, js_name = onCompletion)]
    pub fn on_completion(&mut self, params: JsValue) -> JsValue {
        let CompletionParams {
            text_document_position,
            ..
        } = from_value(params).unwrap();

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
        let mut identifiers = Vec::new();

        if let Ok(pairs) = pairs {
            for pair in pairs.into_iter() {
                if pair.as_rule() == Rule::grammar_rule {
                    let mut inner = pair.into_inner();
                    let inner = inner.next().unwrap();
                    identifiers.push(inner.as_str());
                }
            }
        }

        identifiers.extend(BUILTINS);

        to_value(&CompletionResponse::Array(
            identifiers
                .into_iter()
                .filter(|i| partial_identifier.is_empty() || i.starts_with(partial_identifier))
                .map(|i| CompletionItem {
                    label: i.to_owned(),
                    kind: Some(CompletionItemKind::FIELD),
                    ..Default::default()
                })
                .collect(),
        ))
        .unwrap()
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = PestLanguageServer, js_name = onDocumentFormatting)]
    pub fn on_format(&mut self, params: JsValue) -> JsValue {
        let DocumentFormattingParams { text_document, .. } = from_value(params).unwrap();

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
            return to_value(&vec![TextEdit::new(range, formatted)]).unwrap();
        }

        JsValue::null()
    }

    fn reload(&mut self) -> Diagnostics {
        log("Reloading all diagnostics");
        let mut diagnostics = Diagnostics::new();

        for (url, document) in &self.documents {
            log(&format!("\tReloading diagnostics for {}", url));
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

    fn send_diagnostics(&self, diagnostics: &Diagnostics) {
        let this = &JsValue::null();
        for params in diagnostics.values() {
            let params = &to_value(&params).unwrap();
            if let Err(e) = self.diagnostics_callback.call1(this, params) {
                log(&format!("Error sending diagnostics"));
                log(&format!("{:#?}", e));
            }
        }
    }
}
