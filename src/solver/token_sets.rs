use crate::matches::{Grammar, Group, MatchId, Rule, Term, Token};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum TokenOrGroup {
    Token(Token),
    Group(Group, Rule),
}

pub fn get_set_for_match(
    grammar: &Grammar,
    match_: MatchId,
    start_index: usize,
) -> Vec<TokenOrGroup> {
    let offset = get_match_set_start_index(grammar, start_index, match_);

    let Some(offset) = offset else {
        return vec![];
    };

    let match_ = grammar.get(match_);
    let token_set = match_.terms.iter().skip(offset).map_while(|t| match t {
        Term::Token(token) => Some(TokenOrGroup::Token(*token)),
        Term::Group(group, rule) => Some(TokenOrGroup::Group(*group, *rule)),
        Term::Rule(_) => None,
    });

    token_set.collect()
}

/// Same as `get_set_for_match`, except returns the index of where it starts.
pub fn get_match_set_start_index(
    grammar: &Grammar,
    start_index: usize,
    match_: MatchId,
) -> Option<usize> {
    let match_ = grammar.get(match_);

    let empty_offset = match_
        .terms
        .iter()
        .skip(start_index)
        .position(|term| match term {
            Term::Token(_) => true,
            Term::Group(_, _) => true,
            Term::Rule(_) => false,
        });

    if let Some(empty_offset) = empty_offset {
        Some(start_index + empty_offset)
    } else {
        None
    }
}
