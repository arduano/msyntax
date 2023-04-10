use std::collections::HashMap;

use crate::matches::{Grammar, Rule};

use super::{empty_rules::EmptyRuleSolver, path::MatchIndex, structure::EmptySolverRuleValue};

/// An action describing how to seal a match into a rule. Some matches require
/// extra empty rules to be appended to the end of the match.
#[derive(Debug, Clone)]
pub struct SealAction {
    pub into_rule: Rule,
    pub append_extra: Vec<EmptySolverRuleValue>,
}

#[derive(Debug, Clone)]
pub struct SealRules {
    pub rules: HashMap<MatchIndex, SealAction>,
}

impl SealRules {
    pub fn new(grammar: &Grammar, empty: &EmptyRuleSolver) -> Self {
        let mut rules = HashMap::new();

        for (id, match_) in grammar.iter_matches() {
            for i in 0..=match_.terms.len() {
                let index = MatchIndex::new_at_index(id, i);
                let rule = generate_reduce_action_for_match(index, grammar, empty);
                if let Some(set) = rule {
                    rules.insert(index, set);
                }
            }
        }

        Self { rules }
    }
}

/// Read from the end of the match and generate the reduce action if possible.
fn generate_reduce_action_for_match(
    match_index: MatchIndex,
    grammar: &Grammar,
    empty: &EmptyRuleSolver,
) -> Option<SealAction> {
    let mut emptys_to_append = Vec::new();

    let match_ = grammar.get(match_index.id);

    for i in match_index.index..match_.terms.len() {
        let term = &match_.terms[i];
        if let Some(rule) = term.as_rule() {
            if let Some(empty) = empty.get(*rule) {
                emptys_to_append.push(empty.clone());
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(SealAction {
        into_rule: match_.rule,
        append_extra: emptys_to_append,
    })
}
