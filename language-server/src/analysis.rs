use std::{collections::HashMap, iter};

use pest::iterators::Pairs;
use pest_meta::parser::Rule;
use tower_lsp::lsp_types::Range;

use crate::helpers::{FindReferences, IntoRange};

#[derive(Debug, Clone)]
/// Stores analysis information for a rule.
pub struct RuleAnalysis {
    /// The location of the entire definition of the rule (i.e. `rule = { "hello" }`).
    pub definition_location: Range,
    /// The location of the name definition of the rule.
    pub identifier_location: Range,
    /// The tokens that make up the rule.
    pub tokens: Vec<(String, Range)>,
    /// The rules expression, in [String] form.
    pub expression: String,
    pub expression_range: Range,
    /// The occurrences of the rule, other than its definition.
    pub references: Vec<Range>,
    /// The rules documentation, in markdown.
    pub doc: Option<String>,
}

#[derive(Debug)]
/// Stores analysis information for a document.
pub struct Analysis {
    /// Holds analyses for individual rules.
    /// [RuleAnalysis] is [None] for builtins.
    pub rules: HashMap<String, RuleAnalysis>,
}

impl Analysis {
    pub fn new(pairs: Pairs<Rule>, capacity: Option<usize>) -> Self {
        let mut precending_docs: Option<String> = None;
        let mut rules = match capacity {
            Some(capacity) => HashMap::with_capacity(capacity),
            None => HashMap::new(),
        };

        for pair in pairs
            .clone()
            .filter(|pair| pair.as_rule() == Rule::grammar_rule)
        {
            let current_span = pair.as_span();
            let mut inner_pairs = pair.into_inner();
            let inner = inner_pairs.next().unwrap();

            match (inner.as_rule(), &mut precending_docs) {
                (Rule::line_doc, Some(docs)) => {
                    docs.push_str(inner.into_inner().next().unwrap().as_str());
                    docs.push('\n');
                }

                (Rule::line_doc, _) => {
                    let mut docs = inner.into_inner().next().unwrap().as_str().to_string();
                    docs.push('\n');
                    precending_docs = Some(docs);
                }

                (Rule::identifier, _) => {
                    let mut doc = precending_docs.take();
                    if let Some(doc) = &mut doc {
                        doc.pop();
                    }

                    let expression_pair = inner_pairs
                        .find(|r| r.as_rule() == Rule::expression)
                        .expect("rule should contain expression");
                    let expression = expression_pair.as_str().to_owned();
                    let expression_range = expression_pair.as_span().into_range();
                    let tokens = expression_pair
                        .into_inner()
                        .map(|e| (e.as_str().to_owned(), e.as_span().into_range()))
                        .collect();
                    let references = pairs.clone().find_references(inner.as_span());

                    let definition_location = current_span.into_range();
                    let identifier_location = inner.as_span().into_range();
                    let analisys = RuleAnalysis {
                        identifier_location,
                        definition_location,
                        tokens,
                        expression,
                        expression_range,
                        references,
                        doc,
                    };
                    rules.insert(inner.as_str().to_owned(), analisys);
                }
                _ => {}
            }
        }

        Analysis { rules }
    }

    pub fn unused_rules(&self) -> impl Iterator<Item = (&str, Range)> {
        self.rules.iter().filter_map(|(name, ra)| {
            if ra.references.is_empty()
                && !name.starts_with('_')
                && name != "COMMENT"
                && name != "WHITE_SPACE"
            {
                return Some((name.as_str(), ra.identifier_location));
            }

            None
        })
    }
}

impl RuleAnalysis {
    pub fn references_and_identifier(&self) -> impl Iterator<Item = Range> {
        self.references
            .iter()
            .copied()
            .chain(iter::once(self.identifier_location))
    }
}
