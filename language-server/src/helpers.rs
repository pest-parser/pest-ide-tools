use std::collections::BTreeMap;

use pest::{error::LineColLocation, iterators::Pairs, Span};
use tower_lsp::lsp_types::{
    Location, Position, PublishDiagnosticsParams, Range, TextDocumentItem, Url,
};

use pest_meta::{parser, validator};

pub(crate) type Documents = BTreeMap<Url, TextDocumentItem>;
pub(crate) type Diagnostics = BTreeMap<Url, PublishDiagnosticsParams>;

pub(crate) fn get_empty_diagnostics(
    (uri, doc): (&Url, &TextDocumentItem),
) -> (Url, PublishDiagnosticsParams) {
    "test";
    let params = PublishDiagnosticsParams::new(uri.clone(), vec![], Some(doc.version));
    (uri.clone(), params)
}

pub(crate) trait IntoRange {
    fn into_lsp_range(self) -> Range;
}

impl IntoRange for LineColLocation {
    fn into_lsp_range(self) -> Range {
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
    fn into_lsp_range(self) -> Range {
        let start = self.start_pos().line_col();
        let end = self.end_pos().line_col();
        LineColLocation::Span((start.0, start.1), (end.0, end.1)).into_lsp_range()
    }
}

pub(crate) trait IntoLocation {
    fn into_lsp_location(self, uri: &Url) -> Location;
}

impl IntoLocation for Span<'_> {
    fn into_lsp_location(self, uri: &Url) -> Location {
        Location::new(uri.clone(), self.into_lsp_range())
    }
}

pub(crate) trait FindAllOccurrences<'a> {
    fn find_all_occurrences(self, identifier: &'a str) -> Vec<Span<'a>>;
}

impl<'a> FindAllOccurrences<'a> for Pairs<'a, parser::Rule> {
    fn find_all_occurrences(self, identifier: &'a str) -> Vec<Span<'a>> {
        let mut spans = vec![];

        for pair in self.clone() {
            if pair.as_rule() == parser::Rule::identifier && pair.as_str() == identifier {
                spans.push(pair.as_span());
            }
            let inner = pair.into_inner();
            spans.extend(inner.find_all_occurrences(identifier));
        }

        spans
    }
}

pub(crate) trait IntoRangeWithLine {
    fn into_lsp_range(self, line: u32) -> Range;
}

impl IntoRangeWithLine for std::ops::Range<usize> {
    fn into_lsp_range(self, line: u32) -> Range {
        let start = Position::new(line, self.start as u32);
        let end = Position::new(line, self.end as u32);
        Range::new(start, end)
    }
}

pub(crate) trait FindWord {
    fn get_word_range_at_idx(self, idx: usize) -> std::ops::Range<usize>;
}

impl FindWord for &str {
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

pub(crate) fn parse_pest_grammar(
    grammar: &str,
) -> Result<Pairs<parser::Rule>, Vec<pest::error::Error<parser::Rule>>> {
    let pairs = match parser::parse(parser::Rule::grammar_rules, grammar) {
        Ok(pairs) => Ok(pairs),
        Err(error) => Err(vec![error]),
    }?;

    validator::validate_pairs(pairs.clone())?;
    parser::consume_rules(pairs.clone())?;

    Ok(pairs)
}
