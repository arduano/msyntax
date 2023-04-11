use crate::matches::{MatchId, Rule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptySolverMatchValue {
    pub id: MatchId,
    pub fields: Vec<EmptySolverRuleValue>,
}

impl EmptySolverMatchValue {
    pub fn new(id: MatchId, fields: Vec<EmptySolverRuleValue>) -> Self {
        Self { id, fields }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptySolverRuleValue {
    pub rule: Rule,
    pub match_index: usize,
    pub match_value: EmptySolverMatchValue,
}

impl EmptySolverRuleValue {
    pub fn new(rule: Rule, match_index: usize, match_value: EmptySolverMatchValue) -> Self {
        Self {
            rule,
            match_index,
            match_value,
        }
    }
}

pub struct IncompleteMatch {
    pub id: MatchId,
    pub fields: Vec<EmptySolverRuleValue>,
}
