use crate::matches::{MatchId, Rule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptySolverMatchValue {
    match_: MatchId,
    fields: Vec<EmptySolverRuleValue>,
}

impl EmptySolverMatchValue {
    pub fn new(match_: MatchId, fields: Vec<EmptySolverRuleValue>) -> Self {
        Self { match_, fields }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptySolverRuleValue {
    rule: Rule,
    match_index: usize,
    match_value: EmptySolverMatchValue,
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
