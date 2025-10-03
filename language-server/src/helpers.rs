use std::collections::HashMap;

use pest::{
    Span,
    error::{Error, ErrorVariant, LineColLocation},
    iterators::Pairs,
};
use pest_meta::{
    parser::{self, Rule},
    validator,
};
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, Position, PublishDiagnosticsParams, Range, TextDocumentItem,
    Url,
};
use unicode_segmentation::UnicodeSegmentation;

pub type Documents = HashMap<Url, TextDocumentItem>;
pub type Diagnostics = Vec<PublishDiagnosticsParams>;

pub trait IntoRange {
    fn into_range(self) -> Range;
}

impl IntoRange for LineColLocation {
    fn into_range(self) -> Range {
        match self {
            LineColLocation::Pos((line, col)) => {
                let pos = Position::new(line as u32 - 1, col as u32 - 1);
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

pub trait FindReferences<'a> {
    fn find_references(self, definition: Span<'a>) -> Vec<Range>;
}

impl<'a> FindReferences<'a> for Pairs<'a, Rule> {
    fn find_references(self, definition: Span<'a>) -> Vec<Range> {
        let mut ranges = vec![];

        for pair in self {
            if pair.as_rule() == Rule::identifier
                && pair.as_span() != definition
                && pair.as_str() == definition.as_str()
            {
                ranges.push(pair.as_span().into_range());
            }

            let inner = pair.into_inner();
            ranges.extend(inner.find_references(definition));
        }

        ranges
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
    fn word_range_at_idx(self, idx: usize) -> std::ops::Range<usize>;
}

impl FindWordRange for &str {
    fn word_range_at_idx(self, search_idx: usize) -> std::ops::Range<usize> {
        fn is_not_identifier(c: char) -> bool {
            c.is_whitespace()
                || c == '*'
                || c == '+'
                || c == '?'
                || c == '!'
                || c == '&'
                || c == '~'
                || c == '{'
                || c == '}'
                || c == '['
                || c == ']'
                || c == '('
                || c == ')'
        }

        let next = str_range(self, &(search_idx..self.len()))
            .graphemes(true)
            .enumerate()
            .find(|(_index, char)| is_not_identifier(char.chars().next().unwrap_or(' ')))
            .map(|(index, _char)| index)
            .map(|index| search_idx + index)
            .unwrap_or(self.len());

        let preceding = str_range(self, &(0..search_idx))
            .graphemes(true)
            .rev()
            .enumerate()
            .find(|(_index, char)| is_not_identifier(char.chars().next().unwrap_or(' ')))
            .map(|(index, _char)| index)
            .map(|index| search_idx - index)
            .unwrap_or(0);

        preceding..next
    }
}

/// Returns a string from a range of human characters (graphemes). Respects unicode.
pub fn str_range(s: &str, range: &std::ops::Range<usize>) -> String {
    s.graphemes(true)
        .skip(range.start)
        .take(range.len())
        .collect()
}

pub trait IntoDiagnostics {
    fn into_diagnostics(self) -> Vec<Diagnostic>;
}

impl IntoDiagnostics for Vec<pest::error::Error<Rule>> {
    fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.iter().map(error_diagnostic).collect()
    }
}

fn error_diagnostic(e: &Error<Rule>) -> Diagnostic {
    let message = error_message(e);
    Diagnostic {
        range: e.line_col.clone().into_range(),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("Pest Language Server".to_owned()),
        message,
        ..Default::default()
    }
}

fn error_message(e: &Error<Rule>) -> String {
    match &e.variant {
        ErrorVariant::ParsingError {
            positives,
            negatives,
        } => parsing_error(positives, negatives),
        ErrorVariant::CustomError { message } => message.clone(),
    }
}

fn parsing_error(positives: &[Rule], negatives: &[Rule]) -> String {
    let expected = if !positives.is_empty() {
        let positives = positives
            .iter()
            .map(|s| format!("\"{:#?}\"", s))
            .collect::<Vec<String>>()
            .join(", ");
        format!(" (expected {positives})")
    } else {
        String::new()
    };

    let unexpected = if !negatives.is_empty() {
        let negatives = negatives
            .iter()
            .map(|s| format!("\"{:#?}\"", s))
            .collect::<Vec<String>>()
            .join(", ");
        format!(" (expected {negatives})")
    } else {
        String::new()
    };

    format!("Parsing error{expected}{unexpected}")
}

pub fn validate_pairs(pairs: Pairs<'_, Rule>) -> Result<(), Vec<Error<Rule>>> {
    validator::validate_pairs(pairs.clone())?;
    // This calls validator::validate_ast under the hood
    parser::consume_rules(pairs)?;
    Ok(())
}

pub trait RangeContains {
    fn contains(&self, other: Self) -> bool;
}

impl RangeContains for Range {
    fn contains(&self, other: Self) -> bool {
        self.start.line <= other.start.line
            && self.start.character <= other.start.character
            && self.end.line >= other.end.line
            && self.end.character >= other.end.character
    }
}
