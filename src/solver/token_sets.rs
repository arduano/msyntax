use crate::matches::{Grammar, Group, MatchId, Rule, Term, Token};

use super::empty_rules::EmptyRuleSolver;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum TokenOrGroup {
    Token(Token),
    Group(Group, Rule),
}

pub fn get_set_for_match(
    grammar: &Grammar,
    empty_rules: &EmptyRuleSolver,
    match_: MatchId,
    start_index: usize,
) -> Vec<TokenOrGroup> {
    let mut first_set = Vec::new();

    let match_ = grammar.get(match_);

    let mut has_passed_empty = false;
    for term in match_.terms.iter().skip(start_index) {
        match term {
            Term::Token(token) => {
                first_set.push(TokenOrGroup::Token(*token));
            }
            Term::Group(group, rule) => {
                first_set.push(TokenOrGroup::Group(*group, *rule));
            }
            Term::Rule(rule) => {
                if empty_rules.is_empty(*rule) {
                    let passed_empty_and_passed_tokens = has_passed_empty && !first_set.is_empty();
                    if passed_empty_and_passed_tokens {
                        break;
                    } else {
                        has_passed_empty = true;
                        continue;
                    }
                } else {
                    break;
                }
            }
        }
    }

    first_set
}

/// Same as `get_set_for_match`, except returns the index of where it starts.
pub fn get_match_first_set_index(
    grammar: &Grammar,
    empty_rules: &EmptyRuleSolver,
    match_: MatchId,
) -> Option<usize> {
    let match_ = grammar.get(match_);

    for (i, term) in match_.terms.iter().enumerate() {
        match term {
            Term::Token(_) => {
                return Some(i);
            }
            Term::Group(_, _) => {
                return Some(i);
            }
            Term::Rule(rule) => {
                if empty_rules.is_empty(*rule) {
                    continue;
                } else {
                    return None;
                }
            }
        }
    }

    return None;
}
