use std::collections::HashMap;

use crate::matches::{Grammar, Rule, Term};

use super::{
    empty_rules::EmptyRuleSolver, path::MatchIndex, structure::EmptySolverRuleValue,
    token_sets::TokenOrGroup,
};

#[derive(Debug, Clone)]
pub struct DirectFollowSet {
    pub tokens: Vec<TokenOrGroup>,
    pub append_extra: Vec<EmptySolverRuleValue>,
}

#[derive(Debug, Clone)]
pub struct EnterRuleFollowSet {
    pub rule: Rule,
    pub append_extra: Vec<EmptySolverRuleValue>,
}

#[derive(Debug, Clone)]
pub enum FollowSet {
    Direct(DirectFollowSet),
    Enter(EnterRuleFollowSet),
}

#[derive(Debug, Clone)]
pub struct FollowSets {
    pub sets: HashMap<MatchIndex, Vec<FollowSet>>,
}

impl FollowSets {
    pub fn new(grammar: &Grammar, empty: &EmptyRuleSolver) -> Self {
        let mut sets = HashMap::new();

        for (id, match_) in grammar.iter_matches() {
            for i in 0..match_.terms.len() {
                let index = MatchIndex::new_at_index(id, i);
                let match_sets = generate_set_for_match(index, grammar, empty);
                sets.insert(index, match_sets);
            }
        }

        dbg!(&sets);

        Self { sets }
    }
}

fn generate_set_for_match(
    match_index: MatchIndex,
    grammar: &Grammar,
    empty: &EmptyRuleSolver,
) -> Vec<FollowSet> {
    let mut sets = Vec::new();

    let match_ = grammar.get(match_index.id);

    let mut i = match_index.index;

    let mut emptys_to_append = Vec::new();

    while i < match_.terms.len() {
        let term = &match_.terms[i];

        match term {
            Term::Token(_) | Term::Group(_, _) => {
                // We break and continue in the next loop.
                break;
            }
            Term::Rule(rule) => {
                sets.push(FollowSet::Enter(EnterRuleFollowSet {
                    rule: *rule,
                    append_extra: emptys_to_append.clone(),
                }));

                if let Some(empty) = empty.get(*rule) {
                    emptys_to_append.push(empty.clone());
                } else {
                    break;
                }
            }
        }

        i += 1;
    }

    let mut tokens = Vec::new();
    while i < match_.terms.len() {
        let term = &match_.terms[i];

        match term {
            Term::Token(token) => {
                tokens.push(TokenOrGroup::Token(*token));
            }
            Term::Group(group, rule) => {
                tokens.push(TokenOrGroup::Group(*group, *rule));
            }
            Term::Rule(_) => {
                break;
            }
        }

        i += 1;
    }

    if tokens.len() > 0 {
        sets.push(FollowSet::Direct(DirectFollowSet {
            tokens,
            append_extra: emptys_to_append.clone(),
        }));
    }

    sets
}
