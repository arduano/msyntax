use std::collections::{HashMap, HashSet};

use crate::{
    matches::{Grammar, MatchId, Rule, Term},
    ref_list::ERefList,
    solver::token_sets::get_set_for_match,
};

use super::{
    empty_rules::EmptyRuleSolver,
    path::MatchIndex,
    structure::EmptySolverRuleValue,
    token_sets::{get_match_first_set_index, TokenOrGroup},
};

#[derive(Debug, Clone)]
pub struct PushItem {
    pub id: MatchId,
    pub fields: Vec<EmptySolverRuleValue>,
    pub linked_to_below: bool,
}

#[derive(Debug, Clone)]
pub struct FirstSet {
    pub tokens: Vec<TokenOrGroup>,
    pub then: Vec<PushItem>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StackDisconnect {
    /// The rule that begins the disconnect. In practice, this should be a list of MatchIndex
    /// that point to that rule instead.
    pub parent: Rule,

    /// The child rule, which could
    pub child: Rule,
}

#[derive(Debug, Clone)]
pub struct FirstSets {
    first_sets_per_rule: HashMap<Rule, Vec<FirstSet>>,
    potential_disconnects: Vec<StackDisconnect>,
}

impl FirstSets {
    pub fn new(grammar: &Grammar, empty: &EmptyRuleSolver) -> Self {
        let mut rules = HashMap::new();
        let mut disconnects = HashSet::new();

        for rule in grammar.iter_rules() {
            let matches = calculate_all_destination_matches(grammar, empty, rule);

            let mut sets = Vec::new();

            for (id, paths) in matches {
                let tokens = get_set_for_match(grammar, empty, id, 0);
                let (then, disconnect) =
                    calculate_push_instructions_from_paths(grammar, empty, &paths);

                sets.push(FirstSet { tokens, then });

                if let Some(disconnect) = disconnect {
                    disconnects.insert(disconnect);
                }
            }

            // Sort the sets from largest token lengths to smallest
            sets.sort_by_key(|f| f.tokens.len());
            sets.reverse();

            rules.insert(rule, sets);
        }

        Self {
            first_sets_per_rule: rules,
            potential_disconnects: disconnects.into_iter().collect(),
        }

        // TODO: Allow gathering duplicate first set warnings per rule
    }

    pub fn potential_disconnects(&self) -> &[StackDisconnect] {
        &self.potential_disconnects
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
        let last_match = MatchIndex::new_at_index(next_match, i);
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
        let next_index = &MatchIndex::new_at_index(next_match, i);
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
) -> (Vec<PushItem>, Option<StackDisconnect>) {
    // Instructions that all paths start with (until they diverge)
    let common_start_instructions =
        calculate_common_starts(paths.iter().map(|p| p.iter()).collect());

    // Instructions that all paths end with (until they diverge)
    let mut common_end_instructions =
        calculate_common_starts(paths.iter().map(|p| p.iter().rev()).collect());
    common_end_instructions.reverse();

    if common_start_instructions == common_end_instructions {
        // If the start and end are the same, then we can just return the start
        let push_items = common_start_instructions
            .iter()
            .map(|i| convert_match_index_to_push_instruction(grammar, empty, i, true))
            .collect();

        return (push_items, None);
    }

    let disconnect_parent_rule = if let Some(id) = common_start_instructions.last() {
        // If common start instructions isn't empty, then we use the last rule
        // (We get the rule from the last match, then get the term at its index, then get the rule from the term)
        *grammar.get(id.id).terms[id.index].as_rule().unwrap()
    } else {
        // If common start instructions is empty, then we can just use the first rule
        grammar
            .get(paths.iter().nth(0).unwrap().first().unwrap().id)
            .rule
    };

    let disconnect_child_rule = grammar
        .get(common_end_instructions.first().unwrap().id)
        .rule;

    let disconnect = Some(StackDisconnect {
        parent: disconnect_parent_rule,
        child: disconnect_child_rule,
    });

    let start_iter = common_start_instructions.iter().enumerate().map(|(i, mi)| {
        // Each instruction except for the last one is linked to the next one
        let is_last = i == common_start_instructions.len() - 1;
        convert_match_index_to_push_instruction(grammar, empty, mi, !is_last)
    });

    let end_iter = common_end_instructions
        .iter()
        .map(|mi| convert_match_index_to_push_instruction(grammar, empty, mi, true));

    let push_items = start_iter.chain(end_iter).collect();

    (push_items, disconnect)
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
    linked_to_below: bool,
) -> PushItem {
    let match_ = grammar.get(match_index.id);

    let mut push = PushItem {
        id: match_index.id,
        fields: Vec::new(),
        linked_to_below,
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
