use std::collections::{HashMap, HashSet};

use crate::matches::{Grammar, MatchId, Rule};

pub struct IdenticalGrammarItems {
    match_equal: HashMap<MatchId, HashSet<MatchId>>,
    rule_potential_overlap: HashMap<Rule, HashSet<Rule>>,
}

impl IdenticalGrammarItems {
    fn new(grammar: &Grammar) -> Self {
        let mut sets = Self {
            match_equal: HashMap::new(),
            rule_potential_overlap: HashMap::new(),
        };

        // First, insert all rules and matches with empty hash sets
        for rule in grammar.iter_rules() {
            sets.rule_potential_overlap.insert(rule, HashSet::new());
        }
        for (id, _) in grammar.iter_matches() {
            sets.match_equal.insert(id, HashSet::new());
        }

        // First, we insert directly identical matches
        for (id1, match1) in grammar.iter_matches() {
            for (id2, match2) in grammar.iter_matches() {
                if id1 == id2 {
                    continue;
                }

                if match1.rule != match2.rule {
                    continue;
                }

                if match1.terms.len() != match2.terms.len() {
                    continue;
                }

                let mut identical = true;
                for (term1, term2) in match1.terms.iter().zip(match2.terms.iter()) {
                    if term1 != term2 {
                        identical = false;
                        break;
                    }
                }

                if identical {
                    sets.add_match_equal(id1, id2);
                }
            }
        }

        loop {
            let mut changed = false;

            // Look for rules that might be identical by comparing their matches
            for rule in grammar.iter_rules() {
                let matches = grammar.get_matches_from_rule(rule);
                if matches.len() < 2 {
                    continue;
                }

                for id in matches {
                    let equal_rules = sets
                        .match_iter_equal_rules(*id, grammar)
                        .collect::<HashSet<_>>();

                    for other_rule in equal_rules {
                        if other_rule == rule {
                            continue;
                        }

                        if sets.add_rule_could_be(rule, other_rule) {
                            changed = true;
                        }
                    }
                }
            }

            // Look for matches that could be identical by comparing the inner terms
            for (id1, match1) in grammar.iter_matches() {
                for (id2, match2) in grammar.iter_matches() {
                    if id1 == id2 {
                        continue;
                    }

                    if match1.rule != match2.rule {
                        continue;
                    }

                    if match1.terms.len() != match2.terms.len() {
                        continue;
                    }

                    let mut could_be_equal = true;
                    for (term1, term2) in match1.terms.iter().zip(match2.terms.iter()) {
                        if term1 == term2 {
                            continue;
                        }

                        if let (Some(rule1), Some(rule2)) = (term1.as_rule(), term2.as_rule()) {
                            if !sets.are_rules_overlapping(*rule1, *rule2) {
                                could_be_equal = false;
                                break;
                            }
                        } else {
                            could_be_equal = false;
                            break;
                        }
                    }

                    if could_be_equal {
                        if sets.add_match_equal(id1, id2) {
                            changed = true;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        sets
    }

    fn add_rule_could_be(&mut self, rule: Rule, could_be: Rule) -> bool {
        let left_inserted = self
            .rule_potential_overlap
            .get_mut(&rule)
            .unwrap()
            .insert(could_be);
        let right_inserted = self
            .rule_potential_overlap
            .get_mut(&could_be)
            .unwrap()
            .insert(rule);
        left_inserted || right_inserted
    }

    fn add_match_equal(&mut self, a: MatchId, b: MatchId) -> bool {
        let left_inserted = self.match_equal.get_mut(&a).unwrap().insert(b);
        let right_inserted = self.match_equal.get_mut(&b).unwrap().insert(a);
        left_inserted || right_inserted
    }

    pub fn are_matches_equal(&self, a: MatchId, b: MatchId) -> bool {
        self.match_equal
            .get(&a)
            .map(|s| s.contains(&b))
            .unwrap_or(false)
    }

    pub fn are_rules_overlapping(&self, a: Rule, b: Rule) -> bool {
        self.rule_potential_overlap
            .get(&a)
            .map(|s| s.contains(&b))
            .unwrap_or(false)
    }

    fn match_iter_equal_matches(&self, match_id: MatchId) -> impl Iterator<Item = MatchId> + '_ {
        self.match_equal.get(&match_id).unwrap().iter().copied()
    }

    fn match_iter_equal_rules<'a>(
        &'a self,
        match_id: MatchId,
        grammar: &'a Grammar,
    ) -> impl Iterator<Item = Rule> + '_ {
        self.match_iter_equal_matches(match_id)
            .map(move |match_id| grammar.get(match_id).rule)
    }
}
