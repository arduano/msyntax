use std::collections::{HashMap, HashSet};

use crate::{
    matches::{Grammar, MatchId, Rule, Term},
    ref_list::ERefList,
    solver::token_sets::get_set_for_match,
};

use super::{
    empty_rules::EmptyRuleSolver,
    path::MatchIndex,
    token_sets::{get_match_first_set_index, TokenOrGroup},
    PushItem,
};

#[derive(Debug, Clone)]
pub struct FirstSet {
    pub tokens: Vec<TokenOrGroup>,
    pub then: Vec<PushItem>,
}

#[derive(Debug, Clone)]
pub struct FirstSets {
    first_sets_per_rule: HashMap<Rule, Vec<FirstSet>>,
}

impl FirstSets {
    pub fn new(grammar: &Grammar, empty: &EmptyRuleSolver) -> Self {
        let mut rules = HashMap::new();

        for rule in grammar.iter_rules() {
            let matches = calculate_all_destination_matches(grammar, empty, rule);

            let mut sets = Vec::new();

            for (id, paths) in matches {
                let tokens = get_set_for_match(grammar, empty, id, 0);
                let then = calculate_push_instructions_from_paths(grammar, empty, &paths);
                sets.push(FirstSet { tokens, then });
            }

            rules.insert(rule, sets);
        }

        dbg!(&rules);

        Self {
            first_sets_per_rule: rules,
        }
    }
}

fn calculate_all_destination_matches(
    grammar: &Grammar,
    empty_rules: &EmptyRuleSolver,
    from: Rule,
) -> HashMap<MatchId, HashSet<Vec<MatchIndex>>> {
    let mut destinations = HashMap::new();

    let matches = grammar.get_matches_from_rule(from);

    for &id in matches.iter() {
        recursive_calculate_all_destination_matches(
            grammar,
            empty_rules,
            ERefList::new(),
            id,
            &mut destinations,
        );
    }

    destinations
}

fn recursive_calculate_all_destination_matches(
    grammar: &Grammar,
    empty_rules: &EmptyRuleSolver,
    prev_matches: ERefList<MatchIndex>,
    next_match: MatchId,
    destinations: &mut HashMap<MatchId, HashSet<Vec<MatchIndex>>>,
) {
    // Check if next_match exists in prev_matches, if it doesn then we skip
    for prev_match in prev_matches.iter() {
        if prev_match.id == next_match {
            return;
        }
    }

    if let Some(i) = get_match_first_set_index(grammar, empty_rules, next_match) {
        let last_match = MatchIndex::new_at_index(next_match, i, grammar);
        let full_list = prev_matches.push(&last_match);

        let mut as_vec = full_list.iter().cloned().collect::<Vec<_>>();
        as_vec.reverse();

        destinations
            .entry(next_match)
            .or_insert_with(HashSet::new)
            .insert(as_vec);
    }

    let match_ = grammar.get(next_match);

    for (i, term) in match_.terms.iter().enumerate() {
        let next_index = &MatchIndex::new_at_index(next_match, i, grammar);
        let matches = prev_matches.push(&next_index);

        match term {
            Term::Token(_) => break,
            Term::Group(_, _) => break,
            Term::Rule(rule) => {
                for &id in grammar.get_matches_from_rule(*rule).iter() {
                    recursive_calculate_all_destination_matches(
                        grammar,
                        empty_rules,
                        matches,
                        id,
                        destinations,
                    );
                }

                if !empty_rules.is_empty(*rule) {
                    break;
                }
            }
        }
    }
}

fn calculate_push_instructions_from_paths(
    grammar: &Grammar,
    empty: &EmptyRuleSolver,
    paths: &HashSet<Vec<MatchIndex>>,
) -> Vec<PushItem> {
    // Instructions that all paths start with (until they diverge)
    let common_start_instructions =
        calculate_common_starts(paths.iter().map(|p| p.iter()).collect());

    // Instructions that all paths end with (until they diverge)
    let mut common_end_instructions =
        calculate_common_starts(paths.iter().map(|p| p.iter().rev()).collect());
    common_end_instructions.reverse();

    if common_start_instructions == common_end_instructions {
        // If the start and end are the same, then we can just return the start
        return common_start_instructions
            .iter()
            .map(|i| convert_match_index_to_push_instruction(grammar, empty, i))
            .collect();
    }

    common_start_instructions
        .iter()
        .chain(common_end_instructions.iter())
        .map(|i| convert_match_index_to_push_instruction(grammar, empty, i))
        .collect()
}

/// Return a vector of elements that are common to the starts of all iterators.
fn calculate_common_starts<T: PartialEq>(mut paths: Vec<impl Iterator<Item = T>>) -> Vec<T> {
    let mut common_start = Vec::new();

    if paths.is_empty() {
        return common_start;
    }

    loop {
        let mut all_same = true;
        let mut first = None;

        for path in paths.iter_mut() {
            if let Some(item) = path.next() {
                if let Some(first) = &first {
                    if first != &item {
                        all_same = false;
                        break;
                    }
                } else {
                    first = Some(item);
                }
            } else {
                all_same = false;
                break;
            }
        }

        if all_same {
            common_start.push(first.unwrap());
        } else {
            break;
        }
    }

    common_start
}

fn convert_match_index_to_push_instruction(
    grammar: &Grammar,
    empty_rules: &EmptyRuleSolver,
    match_index: &MatchIndex,
) -> PushItem {
    let match_ = grammar.get(match_index.id);

    let mut push = PushItem {
        id: match_index.id,
        fields: Vec::new(),
    };

    for i in 0..match_index.index {
        let term = &match_.terms[i];

        match term {
            Term::Token(_) => {
                unreachable!();
            }
            Term::Group(_, _) => {
                unreachable!();
            }
            Term::Rule(rule) => {
                if let Some(empty) = empty_rules.get(*rule) {
                    push.fields.push(empty.clone());
                } else {
                    unreachable!();
                }
            }
        }
    }

    push
}
