use crate::{
    matches::{Grammar, Rule, Term},
    ref_list::RefList,
};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("Cyclical dependency found in rule {rule:?}")]
pub struct CyclicalDependencyError {
    rule: Rule,
}

pub fn validate_no_cyclical_dependencies(grammar: &Grammar) -> Result<(), CyclicalDependencyError> {
    for rule in grammar.iter_rules() {
        has_escape_condition(grammar, rule)?;
    }

    Ok(())
}

fn has_escape_condition(grammar: &Grammar, rule: Rule) -> Result<(), CyclicalDependencyError> {
    let prev_rules = RefList::new(&rule);
    if !has_escape_condition_recursive(grammar, prev_rules) {
        return Err(CyclicalDependencyError { rule });
    }

    Ok(())
}

fn has_escape_condition_recursive(grammar: &Grammar, prev_rules: RefList<Rule>) -> bool {
    let rule = *prev_rules.top();
    let matches = grammar.get_matches_from_rule(rule);

    for &match_id in matches {
        let match_ = grammar.get(match_id);

        let first = match_.terms.first();
        match first {
            None => {}
            Some(Term::Token(_)) => return true,
            Some(Term::Rule(rule)) => {
                if prev_rules.contains(rule) {
                    continue;
                }

                if has_escape_condition_recursive(grammar, prev_rules.push(rule)) {
                    return true;
                }
            }
            Some(Term::Group(_, _)) => {}
        }
    }

    false
}
