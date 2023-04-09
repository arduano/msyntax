use std::collections::VecDeque;

use crate::matches::{Grammar, MatchId, Rule};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct MatchIndex {
    pub id: MatchId,
    pub index: usize,
}

impl MatchIndex {
    pub fn new(id: MatchId) -> Self {
        Self { id, index: 0 }
    }

    /// Create a new MatchIndex, except with a different index, pointing to a specific term.
    pub fn new_at_index(id: MatchId, index: usize) -> Self {
        let mut ind = Self::new(id);
        ind.advance_by(index);
        ind
    }

    pub fn advance_by(&mut self, n: usize) {
        self.index += n;
    }
}
