use std::collections::HashMap;

use pest::{Span, iterators::Pairs};
use pest_meta::parser::Rule;
use tower_lsp::lsp_types::Location;
use url::Url;

use crate::helpers::{FindOccurrences, IntoLocation};

#[derive(Debug, Clone)]
/// Stores analysis information for a rule.
pub struct RuleAnalysis {
    /// The location of the entire definition of the rule (i.e. `rule = { "hello" }`).
    pub definition_location: Location,
    /// The location of the name definition of the rule.
    pub identifier_location: Location,
    /// The tokens that make up the rule.
    pub tokens: Vec<(String, Location)>,
    /// The rules expression, in [String] form.
    pub expression: String,
    /// The occurrences of the rule, including its definition.
    pub occurrences: Vec<Location>,
    /// The rules documentation, in markdown.
    pub doc: Option<String>
}

#[derive(Debug)]
/// Stores analysis information for a document.
pub struct Analysis {
    /// The URL of the document that this analysis is for.
    pub doc_url: Url,
    /// Holds analyses for individual rules.
    /// [RuleAnalysis] is [None] for builtins.
    pub rules: HashMap<String, RuleAnalysis>
}

impl Analysis {
    /// Updates this analysis from the given pairs.
    pub fn update_from(&mut self, pairs: Pairs<Rule>) {
        let mut precending_docs = Vec::new();
        for pair in pairs
            .clone()
            .filter(|pair| pair.as_rule() == Rule::grammar_rule)
        {
            let current_span = Some(pair.as_span());
            let mut inner_pairs = pair.into_inner();
            let inner = inner_pairs.next().unwrap();

            match inner.as_rule() {
                Rule::line_doc => {
                    precending_docs.extend(inner.into_inner().next().map(|pair| pair.as_str()))
                }
                Rule::identifier => self.new_rule(
                    &pairs,
                    current_span,
                    &mut precending_docs,
                    inner_pairs,
                    inner
                ),
                _ => {}
            }
        }
    }

    fn new_rule(
        &mut self,
        pairs: &Pairs<'_, Rule>,
        current_span: Option<Span<'_>>,
        precending_docs: &mut Vec<&str>,
        mut inner_pairs: Pairs<'_, Rule>,
        inner: pest::iterators::Pair<'_, Rule>
    ) {
        let expression_pair = inner_pairs
            .find(|r| r.as_rule() == Rule::expression)
            .expect("rule should contain expression");
        let expression = expression_pair.as_str().to_owned();
        let tokens = expression_pair
            .into_inner()
            .map(|e| {
                (
                    e.as_str().to_owned(),
                    e.as_span().into_location(&self.doc_url)
                )
            })
            .collect();
        let occurrences = pairs.find_occurrences(&self.doc_url, inner.as_str());

        let doc = if precending_docs.is_empty() {
            None
        } else {
            Some(precending_docs.join("\n"))
        };
        precending_docs.clear();

        let definition_location = current_span
            .expect("rule should have a defined span")
            .into_location(&self.doc_url);
        let identifier_location = inner.as_span().into_location(&self.doc_url);
        let analisys = RuleAnalysis {
            identifier_location,
            definition_location,
            tokens,
            expression,
            occurrences,
            doc
        };
        self.rules.insert(inner.as_str().to_owned(), analisys);
    }

    pub fn get_unused_rules(&self) -> Vec<(&String, &Location)> {
        self.rules
            .iter()
            .filter_map(|(name, ra)| {
                if let Some(occurence) = ra.occurrences.first()
                    && ra.occurrences.len() == 1
                    && !name.starts_with('_')
                {
                    return Some((name, occurence));
                }

                None
            })
            .collect()
    }
}
