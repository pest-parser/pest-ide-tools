use pest::iterators::Pairs;
use pest_meta::parser::Rule;
use reqwest::Url;
use std::collections::BTreeMap;
use tower_lsp::lsp_types::Location;

use crate::{
    builtins::BUILTINS,
    helpers::{FindOccurrences, IntoLocation},
};

#[derive(Debug)]
/// Stores analysis information for a document.
pub struct Analysis {
    /// The URL of the document that this analysis is for.
    pub doc_url: Url,
    /// Maps rule names to their locations in the document. If the rule is a builtin, the location
    /// will be `None`.
    pub rule_names: BTreeMap<String, Option<Location>>,
    /// Maps rule names to all of their occurrences in the document, including their definition.
    pub rule_occurrences: BTreeMap<String, Vec<Location>>,
    /// Maps rule names to their documentation, in Markdown.
    pub rule_docs: BTreeMap<String, String>,
}

impl Analysis {
    /// Updates this analysis from the given pairs.
    pub fn update_from<'a>(&mut self, pairs: Pairs<'a, Rule>) {
        self.rule_names = BTreeMap::new();
        self.rule_docs = BTreeMap::new();
        self.rule_occurrences = BTreeMap::new();

        for builtin in BUILTINS.iter() {
            self.rule_names.insert(builtin.to_string(), None);
        }

        let mut preceding_docs = Vec::new();

        for pair in pairs.clone() {
            if pair.as_rule() == Rule::grammar_rule {
                let inner = pair.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::line_doc => {
                        preceding_docs.push(inner.into_inner().next().unwrap().as_str());
                    }
                    Rule::identifier => {
                        self.rule_names.insert(
                            inner.as_str().to_owned(),
                            Some(inner.as_span().into_location(&self.doc_url)),
                        );
                        self.rule_occurrences.insert(
                            inner.as_str().to_owned(),
                            pairs.find_occurrences(&self.doc_url, inner.as_str()),
                        );

                        if preceding_docs.len() > 0 {
                            let mut buf = String::new();

                            if preceding_docs.len() == 1 {
                                buf.push_str(preceding_docs.first().unwrap().clone());
                            } else {
                                buf.push_str("- ");
                                buf.push_str(preceding_docs.join("\n- ").as_str());
                            }

                            self.rule_docs.insert(inner.as_str().to_owned(), buf);
                            preceding_docs.clear();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
