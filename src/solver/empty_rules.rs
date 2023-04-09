use std::collections::HashMap;

use crate::matches::{Grammar, Rule};

use super::structure::{EmptySolverMatchValue, EmptySolverRuleValue};

#[derive(Debug, Clone)]
pub struct EmptyRuleSolver {
    empty_rules: HashMap<Rule, EmptySolverRuleValue>,
}

impl EmptyRuleSolver {
    pub fn new(grammar: &Grammar) -> Self {
        let mut empty_rules = HashMap::new();

        // First, find all the directly empty rules.
        for rule in grammar.iter_rules() {
            for &match_id in grammar.get_matches_from_rule(rule) {
                let match_ = grammar.get(match_id);
                if match_.terms.is_empty() {
                    // We make an empty match value
                    let match_value = EmptySolverMatchValue::new(match_id, vec![]);
                    // And make the rule that points to it
                    let rule_value = EmptySolverRuleValue::new(
                        rule,
                        grammar.get_rule_match_index(match_id),
                        match_value,
                    );

                    empty_rules.insert(rule, rule_value);
                }
            }
        }

        // Then, find all the indirectly empty rules.
        let mut changed = true;
        while changed {
            changed = false;
            for rule in grammar.iter_rules() {
                if empty_rules.contains_key(&rule) {
                    continue;
                }

                let mut value = None;
                for &match_id in grammar.get_matches_from_rule(rule) {
                    let match_ = grammar.get(match_id);

                    // Check if all the terms are potentially empty rules.
                    let mut is_empty = true;
                    for term in &match_.terms {
                        if let Some(rule) = term.as_rule() {
                            if !empty_rules.contains_key(&rule) {
                                is_empty = false;
                                break;
                            }
                        } else {
                            is_empty = false;
                            break;
                        }
                    }

                    if is_empty {
                        // If it's empty, we make the match/rule value.
                        let mut fields = vec![];
                        for term in &match_.terms {
                            let rule = term.as_rule().unwrap();
                            let rule_value = empty_rules.get(&rule).unwrap();
                            fields.push(rule_value.clone());
                        }

                        let match_value = EmptySolverMatchValue::new(match_id, fields);
                        value = Some(EmptySolverRuleValue::new(
                            rule,
                            grammar.get_rule_match_index(match_id),
                            match_value,
                        ));

                        break;
                    }
                }

                if let Some(value) = value {
                    empty_rules.insert(rule, value);
                    changed = true;
                }
            }
        }

        Self { empty_rules }
    }

    pub fn get(&self, rule: Rule) -> Option<&EmptySolverRuleValue> {
        self.empty_rules.get(&rule)
    }

    pub fn is_empty(&self, rule: Rule) -> bool {
        self.empty_rules.contains_key(&rule)
    }
}
