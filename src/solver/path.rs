use std::collections::{HashMap, VecDeque};

use crate::matches::{Grammar, MatchId, Rule};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct MatchIndex {
    pub id: MatchId,
    pub rule: Rule,
    pub index_in_rule: usize,
    pub index: usize,
    pub length: usize,
}

impl MatchIndex {
    pub fn new(id: MatchId, grammar: &Grammar) -> Self {
        let match_ = grammar.get(id);
        Self {
            id,
            index: 0,
            rule: match_.rule,
            index_in_rule: grammar.get_rule_match_index(id),
            length: grammar.get(id).terms.len(),
        }
    }

    /// Create a new MatchIndex, except with a different index, pointing to a specific term.
    pub fn new_at_index(id: MatchId, index: usize, grammar: &Grammar) -> Self {
        let mut ind = Self::new(id, grammar);
        assert!(index < ind.length);
        ind.advance_by(index);
        ind
    }

    pub fn advance_by(&mut self, n: usize) {
        self.index += n;
    }

    pub fn is_done(&self) -> bool {
        self.index >= self.length
    }
}

#[derive(Debug, Clone)]
pub struct SolverPath {
    location: VecDeque<MatchIndex>,
}

impl SolverPath {
    pub fn new(id: MatchId, grammar: &Grammar) -> Self {
        Self {
            location: VecDeque::from(vec![MatchIndex::new(id, grammar)]),
        }
    }

    /// Return true if any MatchIndex appears more than once in the location.
    // FIXME: Delete
    // pub fn is_recursing(&self) -> bool {
    //     let mut seen = HashMap::new();
    //     for index in &self.location {
    //         if seen.contains_key(index) {
    //             return true;
    //         }
    //         seen.insert(index, ());
    //     }
    //     false
    // }

    pub fn prepend_match(&mut self, id: MatchId, inner_match_index: usize, grammar: &Grammar) {
        let match_ = grammar.get(id);
        let tail = self.location.front().unwrap();
        if match_.terms[inner_match_index].as_rule() != Some(&tail.rule) {
            panic!("SolverHead::prepend_match: rule mismatch");
        }
        self.location
            .push_front(MatchIndex::new_at_index(id, inner_match_index, grammar));
    }
}
