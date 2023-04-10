use thiserror::Error;

use crate::matches::{Grammar, MatchId};

use self::{
    cyclical::{validate_no_cyclical_dependencies, CyclicalDependencyError},
    empty_rules::EmptyRuleSolver,
    first_sets::FirstSets,
    follow_sets::FollowSets,
    inner_relations::InnerRelations,
    seal_rules::SealRules,
    structure::EmptySolverRuleValue,
    wrap_sets::WrapSets,
};

mod cyclical;
mod empty_rules;
mod first_sets;
mod follow_sets;
mod identical_check;
mod inner_relations;
mod path;
mod seal_rules;
mod structure;
mod token_sets;
mod wrap_sets;

#[derive(Debug, Error)]
pub enum GrammarError {
    #[error("Cyclical dependency found: {0:?}")]
    Cyclical(#[from] CyclicalDependencyError),
}

pub struct GrammarSolver {
    empty_rules: EmptyRuleSolver,
    inner_relations: InnerRelations,
}

impl GrammarSolver {
    pub fn new(grammar: &Grammar) -> Result<Self, GrammarError> {
        validate_no_cyclical_dependencies(grammar)?;

        let empty_rules = EmptyRuleSolver::new(grammar);
        let inner_relations = InnerRelations::new(grammar);
        let first_sets = FirstSets::new(grammar, &empty_rules);
        let follow_sets = FollowSets::new(grammar, &empty_rules);
        let wrap_sets = WrapSets::new(grammar, &empty_rules, &first_sets);
        let reduce_sets = SealRules::new(grammar, &empty_rules, );

        Ok(Self {
            empty_rules,
            inner_relations,
        })
    }
}

// fn build_tokens_for_match(grammar: &Grammar) -> HashMap<MatchId, Vec<TokenOrGroup>> {
//     let mut tokens_for_match = HashMap::new();

//     for (match_id, match_) in grammar.iter_matches() {
//         let mut tokens = Vec::new();

//         for term in &match_.terms {
//             match term {
//                 Term::Token(token) => tokens.push(TokenOrGroup::Token(*token)),
//                 Term::Group(group, rule) => tokens.push(TokenOrGroup::Group(*group, *rule)),
//                 Term::Rule(_) => break,
//             }
//         }

//         tokens_for_match.insert(match_id, tokens);
//     }

//     dbg!(&tokens_for_match);

//     tokens_for_match
// }

// struct RuleEntry {
//     tokens: Vec<TokenOrGroup>,
//     id: MatchId,
// }

// fn build_possible_match_entries(grammar: &Grammar) -> HashMap<Rule, Vec> {
//     let mut tokens_for_match = HashMap::new();

//     for (match_id, match_) in grammar.iter_matches() {
//         let mut tokens = Vec::new();

//         for term in &match_.terms {
//             match term {
//                 Term::Token(token) => tokens.push(TokenOrGroup::Token(*token)),
//                 Term::Group(group, rule) => tokens.push(TokenOrGroup::Group(*group, *rule)),
//                 Term::Rule(_) => break,
//             }
//         }

//         tokens_for_match.insert(match_id, tokens);
//     }

//     dbg!(&tokens_for_match);

//     tokens_for_match
// }
