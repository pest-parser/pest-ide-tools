use pest::{iterators::Pairs, Span};
use pest_meta::parser::Rule;
use reqwest::Url;
use std::collections::HashMap;
use tower_lsp::lsp_types::Location;

use crate::{
    builtins::BUILTINS,
    helpers::{FindOccurrences, IntoLocation},
};

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
    pub doc: String,
}

#[derive(Debug)]
/// Stores analysis information for a document.
pub struct Analysis {
    /// The URL of the document that this analysis is for.
    pub doc_url: Url,
    /// Holds analyses for individual rules.
    /// [RuleAnalysis] is [None] for builtins.
    pub rules: HashMap<String, Option<RuleAnalysis>>,
}

impl Analysis {
    /// Updates this analysis from the given pairs.
    pub fn update_from(&mut self, pairs: Pairs<Rule>) {
        self.rules = HashMap::new();

        for builtin in BUILTINS.iter() {
            self.rules.insert(builtin.to_string(), None);
        }

        let mut preceding_docs = Vec::new();
        let mut current_span: Option<Span>;

        for pair in pairs.clone() {
            if pair.as_rule() == Rule::grammar_rule {
                current_span = Some(pair.as_span());
                let mut inner_pairs = pair.into_inner();
                let inner = inner_pairs.next().unwrap();

                match inner.as_rule() {
                    Rule::line_doc => {
                        preceding_docs.push(inner.into_inner().next().unwrap().as_str());
                    }
                    Rule::identifier => {
                        let expression_pair = inner_pairs
                            .find(|r| r.as_rule() == Rule::expression)
                            .expect("rule should contain expression");
                        let expression = expression_pair.as_str().to_owned();
                        let expressions = expression_pair
                            .into_inner()
                            .map(|e| {
                                (
                                    e.as_str().to_owned(),
                                    e.as_span().into_location(&self.doc_url),
                                )
                            })
                            .collect();
                        let occurrences = pairs.find_occurrences(&self.doc_url, inner.as_str());
                        let mut docs = None;

                        if !preceding_docs.is_empty() {
                            let mut buf = String::new();

                            if preceding_docs.len() == 1 {
                                buf.push_str(preceding_docs.first().unwrap());
                            } else {
                                buf.push_str("- ");
                                buf.push_str(preceding_docs.join("\n- ").as_str());
                            }

                            docs = Some(buf);
                            preceding_docs.clear();
                        }

                        self.rules.insert(
                            inner.as_str().to_owned(),
                            Some(RuleAnalysis {
                                identifier_location: inner.as_span().into_location(&self.doc_url),
                                definition_location: current_span
                                    .expect("rule should have a defined span")
                                    .into_location(&self.doc_url),
                                tokens: expressions,
                                expression,
                                occurrences,
                                doc: docs.unwrap_or_else(|| "".to_owned()),
                            }),
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn get_unused_rules(&self) -> Vec<(&String, &Location)> {
        self.rules
            .iter()
            .filter(|(_, ra)| {
                if let Some(ra) = ra {
                    ra.occurrences.len() == 1
                } else {
                    false
                }
            })
            .filter(|(name, _)| !BUILTINS.contains(&name.as_str()) && !name.starts_with('_'))
            .map(|(name, ra)| {
                (
                    name,
                    ra.as_ref().unwrap().occurrences.first().unwrap_or_else(|| {
                        panic!("Expected at least one occurrence for rule {}", name)
                    }),
                )
            })
            .collect()
    }
}
