use std::collections::HashMap;

use crate::{
    matches::{Grammar, MatchId, Rule, Term},
    ref_list::RefList,
};

type WrapsPerMatch = HashMap<MatchId, HashMap<Rule, Vec<MatchId>>>;

pub struct InnerRelations {
    /// The wraps for each match, which are used to bubble up the value of a match.
    /// Each time, the match gets wrapped into its rule, then the rule gets wrapped in another
    /// match listed in the array, then wrapped in its rule, and so on.
    wraps_per_match: WrapsPerMatch,
}

impl InnerRelations {
    pub fn new(grammar: &Grammar) -> Self {
        let wraps_per_match = calc_wraps_per_match(grammar);

        Self { wraps_per_match }
    }
}

fn calc_wraps_per_match(grammar: &Grammar) -> WrapsPerMatch {
    let mut wraps_per_match = HashMap::new();

    for (match_id, _) in grammar.iter_matches() {
        calc_wraps_per_match_recursive(grammar, RefList::new(&match_id), &mut wraps_per_match);
    }

    wraps_per_match
}

fn calc_wraps_per_match_recursive(
    grammar: &Grammar,
    matches: RefList<MatchId>,
    wraps_per_match: &mut WrapsPerMatch,
) {
    let rule = grammar.get(*matches.top()).rule;

    // Insert the rule into the map if it's not there yet.
    let mut wraps: Vec<_> = matches.iter().cloned().collect();
    wraps.pop();
    wraps.reverse();

    wraps_per_match
        .entry(*matches.base())
        .or_insert_with(HashMap::new)
        .insert(rule, wraps);

    // Recurse into other matches that contain this rule as the sole element.
    for (match_id, match_) in grammar.iter_matches() {
        if match_.terms.len() == 1 {
            if match_.terms[0] == Term::Rule(rule) {
                if matches.contains(&match_id) {
                    continue;
                }

                calc_wraps_per_match_recursive(grammar, matches.push(&match_id), wraps_per_match);
            }
        }
    }
}
