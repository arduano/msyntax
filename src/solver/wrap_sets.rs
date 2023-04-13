use std::collections::{HashMap, HashSet};

use crate::{
    matches::{Grammar, MatchId, Rule, Term},
    ref_list::ERefList,
};

use super::{
    empty_rules::EmptyRuleSolver, first_sets::FirstSets, path::MatchIndex,
    structure::EmptySolverRuleValue,
};

/// Describes a single step in wrapping an item and bubbling it up into a parent rule.
#[derive(Debug, Clone)]
pub struct EmptyWrapAction {
    pub match_id: MatchId,

    // The empty values to the left side of the match
    pub left_empty: Vec<EmptySolverRuleValue>,
    // The empty values to the right side of the match
    pub right_empty: Vec<EmptySolverRuleValue>,
}

#[derive(Debug, Clone)]
pub struct InsertAction {
    // If the parent matches, then wrap using the following matches:
    pub wrap_actions: Vec<EmptyWrapAction>,
    // And then append the child rule too.
}

#[derive(Debug, Clone)]
pub struct WrapAction {
    // If matches the following:
    pub if_matches: MatchIndex,

    // Then wrap it using the following actions:
    pub wrap_actions: Vec<EmptyWrapAction>,

    // And then become the match in the index, with the following empty fields:
    pub append_empty: Vec<EmptySolverRuleValue>,
    // And then append the child rule too.
}

/// The context of potential wrap actions. All wrap actions depend on the
/// current child rule, as well as the potential rule it might try to become later.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct WrapContext {
    pub parent: Rule,
    pub child: Rule,
}

/// The potential actions that a rule can use to wrap based on the context. First, it should
/// check if any of the wrap actions fit. Then, if insert_action is Some, it should check
/// whether the parent match match can be inserted into and it will follow the insert
/// actions to insert it.
#[derive(Debug, Clone)]
pub struct WrapData {
    pub wrap_actions: Vec<WrapAction>,
    pub insert_action: Option<InsertAction>,
}

pub struct WrapSets {
    pub sets: HashMap<WrapContext, WrapData>,
}

impl WrapSets {
    pub fn new(grammar: &Grammar, empty: &EmptyRuleSolver, first_sets: &FirstSets) -> Self {
        // A map of disconnects from the parent to a set of each child it could have.
        // It starts with the initial values, but more children are discovered as the
        // wrap sets are calculated.
        let mut potential_disconnects = HashMap::new();

        // Populate the initial potential disconnects map.
        for disconnect in &first_sets.potential_disconnects {
            potential_disconnects
                .entry(disconnect.parent)
                .or_insert_with(HashSet::new)
                .insert(disconnect.child);
        }

        let mut sets = HashMap::new();

        let mut changed = true;
        while changed {
            changed = false;

            for (parent, children) in potential_disconnects.clone().iter() {
                for child in children {
                    let ctx = WrapContext {
                        parent: *parent,
                        child: *child,
                    };

                    if sets.contains_key(&ctx) {
                        continue;
                    }

                    // If it's absent, then calculate and insert it
                    let data = get_wrap_data_for(&ctx, grammar, empty);

                    // Scan the generated wrap actions for any new potential disconnects
                    for wrap_action in &data.wrap_actions {
                        let child_id = wrap_action.if_matches.id;
                        let child_rule = grammar.get(child_id).rule;
                        let inserted = potential_disconnects
                            .entry(*parent)
                            .or_insert_with(HashSet::new)
                            .insert(child_rule);

                        if inserted {
                            changed = true;
                        }
                    }

                    sets.insert(ctx, data);
                }
            }
        }

        Self { sets }
    }
}

#[derive(Debug, Clone)]
struct WrapDataBuilder {
    wrap_actions: HashMap<MatchIndex, Vec<WrapAction>>,
    insert_actions: Vec<InsertAction>,
}

/// Gets the wrap data, as well as other potential children to check for that parent.
fn get_wrap_data_for(ctx: &WrapContext, grammar: &Grammar, empty: &EmptyRuleSolver) -> WrapData {
    let mut builder = WrapDataBuilder {
        wrap_actions: HashMap::new(),
        insert_actions: Vec::new(),
    };

    if ctx.parent == ctx.child {
        // If the parent and child are the same, then we we can just add a direct insert action.
        builder.insert_actions.push(InsertAction {
            wrap_actions: Vec::new(),
        });
    }

    recursive_calculate_all_destination_matches_for_rule(
        grammar,
        empty,
        ERefList::new(),
        ctx.clone(),
        ctx.parent,
        ctx.child,
        &mut builder,
    );

    WrapData {
        insert_action: pick_best_insert_action(builder.insert_actions),
        wrap_actions: builder
            .wrap_actions
            .into_iter()
            .map(|(_, a)| pick_best_wrap_action(a))
            .collect(),
    }
}

#[derive(Debug, Clone, Copy)]
struct RecursiveWrap {
    index: MatchIndex,
}

fn recursive_calculate_all_destination_matches_for_rule(
    grammar: &Grammar,
    empty_rules: &EmptyRuleSolver,
    prev_matches: ERefList<RecursiveWrap>,
    ctx: WrapContext,
    next_rule: Rule,
    target_rule: Rule,
    data: &mut WrapDataBuilder,
) {
    for &id in grammar.get_matches_from_rule(next_rule).iter() {
        recursive_calculate_all_destination_matches_for_match(
            grammar,
            empty_rules,
            prev_matches,
            ctx,
            id,
            target_rule,
            data,
        );
    }
}

fn recursive_calculate_all_destination_matches_for_match(
    grammar: &Grammar,
    empty_rules: &EmptyRuleSolver,
    prev_matches: ERefList<RecursiveWrap>,
    ctx: WrapContext,
    next_match: MatchId,
    target_rule: Rule,
    data: &mut WrapDataBuilder,
) {
    let match_ = grammar.get(next_match);

    for (i, term) in match_.terms.iter().enumerate() {
        match term {
            Term::Rule(rule) => {
                let match_index = MatchIndex {
                    id: next_match,
                    index: i,
                };

                let recursive_wrap = RecursiveWrap { index: match_index };

                let next_matches = prev_matches.push(&recursive_wrap);

                if !recursive_wraps_contain_match(prev_matches, next_match) {
                    recursive_calculate_all_destination_matches_for_rule(
                        grammar,
                        empty_rules,
                        next_matches,
                        ctx,
                        *rule,
                        target_rule,
                        data,
                    );
                }

                if *rule == target_rule {
                    extend_builder_from_matches(grammar, empty_rules, next_matches, data);
                }
            }
            Term::Group(_, _) | Term::Token(_) => break,
        }
    }
}

fn recursive_wraps_contain_match(wraps: ERefList<RecursiveWrap>, match_: MatchId) -> bool {
    for wrap in wraps.iter() {
        if wrap.index.id == match_ {
            return true;
        }
    }

    false
}

fn are_terms_empty(terms: &[Term], grammar: &EmptyRuleSolver) -> bool {
    for term in terms.iter() {
        match term {
            Term::Rule(rule) => {
                if !grammar.is_empty(*rule) {
                    return false;
                }
            }
            Term::Group(_, _) | Term::Token(_) => return false,
        }
    }

    true
}

/// Extend the builder with the current RecursiveWrap list
fn extend_builder_from_matches(
    grammar: &Grammar,
    empty_rules: &EmptyRuleSolver,
    prev_matches: ERefList<RecursiveWrap>,
    data: &mut WrapDataBuilder,
) {
    let mut empty_wrap_actions = Vec::new();

    let mut full_list_empty = true;
    for wrap in prev_matches.iter() {
        let match_ = grammar.get(wrap.index.id);

        // If there's more terms after this one, then we can make a wrap action
        if wrap.index.index < match_.terms.len() - 1 {
            let wrap = WrapAction {
                if_matches: MatchIndex {
                    id: wrap.index.id,
                    index: wrap.index.index + 1,
                },
                wrap_actions: empty_wrap_actions.clone(),
                append_empty: match_.terms[0..wrap.index.index]
                    .iter()
                    .map(|term| empty_rules.get(*term.as_rule().unwrap()).unwrap().clone())
                    .collect(),
            };

            data.wrap_actions
                .entry(wrap.if_matches)
                .or_insert_with(Vec::new)
                .push(wrap);
        }

        let left_terms = &match_.terms[0..wrap.index.index];
        let right_terms = &match_.terms[wrap.index.index + 1..];
        let can_empty_wrap =
            are_terms_empty(left_terms, empty_rules) && are_terms_empty(right_terms, empty_rules);

        if can_empty_wrap {
            // Map the fields to the empty values

            let terms_into_emptys = |terms: &[Term]| -> Vec<EmptySolverRuleValue> {
                terms
                    .iter()
                    .map(|term| empty_rules.get(*term.as_rule().unwrap()).unwrap().clone())
                    .collect()
            };

            let left_empty = terms_into_emptys(left_terms);
            let right_empty = terms_into_emptys(right_terms);

            let wrap_action = EmptyWrapAction {
                match_id: wrap.index.id,
                left_empty,
                right_empty,
            };

            empty_wrap_actions.push(wrap_action);
        } else {
            full_list_empty = false;
            break;
        }
    }

    if full_list_empty {
        let insert_action = InsertAction {
            wrap_actions: empty_wrap_actions,
        };

        data.insert_actions.push(insert_action);
    }
}

pub fn pick_best_insert_action(actions: Vec<InsertAction>) -> Option<InsertAction> {
    pick_best_item(actions.into_iter(), |a| a.wrap_actions.len() as u32)
}

pub fn pick_best_wrap_action(actions: Vec<WrapAction>) -> WrapAction {
    pick_best_item(actions.into_iter(), |a| {
        // We want the shallowest wrap actions.
        let depth_score = a.wrap_actions.len() as u32 * 100;

        // For each wrap, we want the least amout of empty items to be added.
        let wrap_empty_score: u32 = a
            .wrap_actions
            .iter()
            .map(|w| w.left_empty.len() as u32 + w.right_empty.len() as u32)
            .sum();

        // If there are multiple equal shallow wrap actions, we want the one with the least
        // empty values inserted.
        let append_score = a.append_empty.len() as u32;

        depth_score + wrap_empty_score + append_score
    })
    .unwrap()
}

/// Pick the item with the smallest score.
pub fn pick_best_item<T>(
    items: impl Iterator<Item = T>,
    mut f: impl FnMut(&T) -> u32,
) -> Option<T> {
    let mut best_item = None;
    let mut best_score = 0;

    for item in items {
        let score = f(&item);

        if score < best_score || best_item.is_none() {
            best_item = Some(item);
            best_score = score;
        }
    }

    best_item
}
