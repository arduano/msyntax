// use std::collections::{HashMap, HashSet};

// use crate::matches::{Grammar, Group, MatchId, Rule, Term, Token};

// #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
// pub enum TokenOrGroup {
//     Token(Token),
//     Group(Group, Rule),
// }

// pub struct MatchAnalysis {
//     /// For any given match, list the tokens that it starts with (if a match starts
//     /// with another match, it's not counted).
//     /// E.g.
//     /// ```
//     /// Paren -> ( Expr )
//     /// Lambda -> ( Args ) => Expr
//     /// ```
//     /// Tokens for `Paren` are the group `()`
//     /// Tokens for `Lambda` are the group `()` and the token `=>`
//     tokens_for_match: HashMap<MatchId, Vec<TokenOrGroup>>,

//     /// For any given rule, list the inner matches that it internally starts with.
//     /// In some case, a match can start with another empty match, then other matches
//     /// are considered.
//     /// E.g.
//     /// ```
//     /// Add -> Expr + Expr
//     /// Expr -> Num | Paren | Lambda
//     /// Lambda -> ( Args ) => Group
//     /// Paren -> ( Expr )
//     /// Num -> r"[0-9]+"
//     /// ```
//     /// Inner matches for `Add` are `Expr`, `Num` and `Paren`
//     inner_matches_for_rule: HashMap<Rule, HashSet<MatchId>>,
// }

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

// fn build_inner_matches_for_match(grammar: &Grammar) -> HashMap<MatchId, HashSet<MatchId>> {
//     // For any given match, list which other matches start with it.
//     // This is useful for walking back up the graph later on.
//     // E.g.
//     // ```
//     // Add -> Expr + Expr
//     // Expr -> Num | Paren | Lambda
//     // Lambda -> ( Args ) => Group
//     // Paren -> ( Expr )
//     // Num -> r"[0-9]+"
//     // ```
//     // For `Expr` the inverse inner expressions are `Add`, for `Lambda` it's `Expr`
//     let mut inner_rules_for_rule_inverse_1_depth = HashMap::<Rule, HashSet<Rule>>::new();

//     for (_, match_) in grammar.iter_matches() {
//         let match_rule = match_.rule;

//         for term in &match_.terms {
//             match term {
//                 Term::Rule(rule) => {
//                     if *rule != match_rule {
//                         let matches = grammar.get_matches_from_rule(*rule);

//                         // Insert them accordingly
//                         for match_ in matches {
//                             let rule = grammar.get(match_).rule;
//                             let inner_rules = inner_rules_for_rule_inverse_1_depth
//                                 .entry(rule)
//                                 .or_insert_with(HashSet::new);
//                             inner_rules.insert(match_rule);
//                         }
//                     }

//                     // TODO: If the term could be empty, don't break
//                     break;
//                 }
//                 Term::Token(_) | Term::Group(_, _) => break,
//             }
//         }
//     }

//     dbg!(&inner_rules_for_rule_inverse_1_depth);

//     let mut inner_matches_for_match = HashMap::new();

//     inner_matches_for_match
// }

// impl MatchAnalysis {
//     pub fn build_for(grammar: &Grammar) -> Self {
//         Self {
//             tokens_for_match: build_tokens_for_match(grammar),
//             inner_matches_for_rule: build_inner_matches_for_match(grammar),
//         }
//     }
// }
