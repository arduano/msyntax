use thiserror::Error;

use crate::matches::{Grammar, Match, MatchId, Rule};

use self::{
    cyclical::{validate_no_cyclical_dependencies, CyclicalDependencyError},
    empty_rules::EmptyRuleSolver,
    first_sets::FirstSets,
    follow_sets::FollowSets,
    seal_rules::SealRules,
    wrap_sets::WrapSets,
};

mod cyclical;
mod empty_rules;
mod first_sets;
mod follow_sets;
mod identical_check;
mod path;
mod seal_rules;
mod structure;
mod token_sets;
mod wrap_sets;

pub use first_sets::FirstSet;
pub use follow_sets::FollowSet;
pub use path::MatchIndex;
pub use seal_rules::SealAction;
pub use structure::EmptySolverRuleValue;
pub use token_sets::TokenOrGroup;
pub use wrap_sets::{EmptyWrapAction, InsertAction, WrapAction, WrapContext, WrapData};

#[derive(Debug, Error)]
pub enum GrammarError {
    #[error("Cyclical dependency found: {0:?}")]
    Cyclical(#[from] CyclicalDependencyError),
}

pub struct GrammarSolver {
    grammar: Grammar,
    empty_rules: EmptyRuleSolver,
    first_sets: FirstSets,
    follow_sets: FollowSets,
    wrap_sets: WrapSets,
    seal_rules: SealRules,
}

impl GrammarSolver {
    pub fn new(grammar: Grammar) -> Result<Self, GrammarError> {
        validate_no_cyclical_dependencies(&grammar)?;

        let empty_rules = EmptyRuleSolver::new(&grammar);
        let first_sets = FirstSets::new(&grammar, &empty_rules);
        let follow_sets = FollowSets::new(&grammar, &empty_rules);
        let wrap_sets = WrapSets::new(&grammar, &empty_rules, &first_sets);
        let seal_rules = SealRules::new(&grammar, &empty_rules);

        Ok(Self {
            grammar,
            empty_rules,
            first_sets,
            follow_sets,
            wrap_sets,
            seal_rules,
        })
    }

    pub fn first_set_for_rule(&self, rule: Rule) -> &[FirstSet] {
        &self
            .first_sets
            .first_sets_per_rule
            .get(&rule)
            .map(|s| s.as_slice())
            .unwrap_or(&[])
    }

    pub fn follow_set_for_match(&self, mi: MatchIndex) -> &[FollowSet] {
        self.follow_sets
            .sets
            .get(&mi)
            .map(|s| s.as_slice())
            .unwrap_or(&[])
    }

    pub fn root_rule(&self) -> Rule {
        Rule::S
    }

    pub fn get_match_rule(&self, id: MatchId) -> Rule {
        self.grammar.get(id).rule
    }

    pub fn get_match(&self, id: MatchId) -> &Match {
        self.grammar.get(id)
    }

    pub fn get_seal_action_for_match(&self, id: MatchIndex) -> Option<&SealAction> {
        self.seal_rules.rules.get(&id)
    }

    pub fn get_wrap_data(&self, parent: Rule, child: Rule) -> Option<&WrapData> {
        self.wrap_sets.sets.get(&WrapContext { parent, child })
    }
}
