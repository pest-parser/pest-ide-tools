use std::collections::HashMap;

use pest::{
    error::{ErrorVariant, LineColLocation},
    iterators::Pairs,
    Span,
};
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, Location, Position, PublishDiagnosticsParams, Range,
    TextDocumentItem, Url,
};

use pest_meta::parser::{self, Rule};

pub type Documents = HashMap<Url, TextDocumentItem>;
pub type Diagnostics = HashMap<Url, PublishDiagnosticsParams>;

pub fn create_empty_diagnostics(
    (uri, doc): (&Url, &TextDocumentItem),
) -> (Url, PublishDiagnosticsParams) {
    let params = PublishDiagnosticsParams::new(uri.clone(), vec![], Some(doc.version));
    (uri.clone(), params)
}

pub trait IntoRange {
    fn into_range(self) -> Range;
}

impl IntoRange for LineColLocation {
    fn into_range(self) -> Range {
        match self {
            LineColLocation::Pos((line, col)) => {
                let pos = Position::new(line as u32, col as u32);
                Range::new(pos, pos)
            }
            LineColLocation::Span((start_line, start_col), (end_line, end_col)) => Range::new(
                Position::new(start_line as u32 - 1, start_col as u32 - 1),
                Position::new(end_line as u32 - 1, end_col as u32 - 1),
            ),
        }
    }
}

impl IntoRange for Span<'_> {
    fn into_range(self) -> Range {
        let start = self.start_pos().line_col();
        let end = self.end_pos().line_col();
        LineColLocation::Span((start.0, start.1), (end.0, end.1)).into_range()
    }
}

pub trait IntoLocation {
    fn into_location(self, uri: &Url) -> Location;
}

impl IntoLocation for Span<'_> {
    fn into_location(self, uri: &Url) -> Location {
        Location::new(uri.clone(), self.into_range())
    }
}

pub trait FindOccurrences<'a> {
    fn find_occurrences(&self, doc_uri: &Url, identifier: &'a str) -> Vec<Location>;
}

impl<'a> FindOccurrences<'a> for Pairs<'a, parser::Rule> {
    fn find_occurrences(&self, doc_uri: &Url, identifier: &'a str) -> Vec<Location> {
        let mut locs = vec![];

        for pair in self.clone() {
            if pair.as_rule() == parser::Rule::identifier && pair.as_str() == identifier {
                locs.push(pair.as_span().into_location(doc_uri));
            }

            let inner = pair.into_inner();
            locs.extend(inner.find_occurrences(doc_uri, identifier));
        }

        locs
    }
}

pub trait IntoRangeWithLine {
    fn into_range(self, line: u32) -> Range;
}

impl IntoRangeWithLine for std::ops::Range<usize> {
    fn into_range(self, line: u32) -> Range {
        let start = Position::new(line, self.start as u32);
        let end = Position::new(line, self.end as u32);
        Range::new(start, end)
    }
}

pub trait FindWordRange {
    fn get_word_range_at_idx(self, idx: usize) -> std::ops::Range<usize>;
}

impl FindWordRange for &str {
    fn get_word_range_at_idx(self, search_idx: usize) -> std::ops::Range<usize> {
        fn is_identifier(c: &char) -> bool {
            !(c.is_whitespace()
                || *c == '*'
                || *c == '+'
                || *c == '?'
                || *c == '!'
                || *c == '&'
                || *c == '~'
                || *c == '{'
                || *c == '}'
                || *c == '['
                || *c == ']'
                || *c == '('
                || *c == ')')
        }

        let next = self[search_idx..]
            .chars()
            .enumerate()
            .find(|(_index, char)| !is_identifier(char))
            .map(|(index, _char)| index)
            .map(|index| search_idx + index)
            .unwrap_or(self.len());

        let preceding = self[0..search_idx]
            .chars()
            .rev()
            .enumerate()
            .find(|(_index, char)| !is_identifier(char))
            .map(|(index, _char)| index)
            .map(|index| search_idx - index)
            .unwrap_or(0);

        preceding..next
    }
}

pub trait IntoDiagnostics {
    fn into_diagnostics(self) -> Vec<Diagnostic>;
}

impl IntoDiagnostics for Vec<pest::error::Error<Rule>> {
    fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.iter()
            .map(|e| {
                Diagnostic::new(
                    e.line_col.clone().into_range(),
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
                                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                            }
                        }
                    },
                    None,
                    None,
                )
            })
            .collect()
    }
}
