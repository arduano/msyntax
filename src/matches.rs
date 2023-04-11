use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Rule {
    S,
    Expr,
    Add,
    Mul,
    Term,
    LParen,
    RParen,

    Static,
    Modifier,
    Vis,
    VisModifier,
    Struct,
    Fn,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Token {
    Num,
    Plus,
    Star,
    Pub,
    Fn,
    Struct,
    Crate,
    Eof,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Group {
    Parens,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Term {
    Rule(Rule),
    Token(Token),
    Group(Group, Rule),
}

impl Term {
    pub fn as_rule(&self) -> Option<&Rule> {
        match self {
            Term::Rule(rule) => Some(rule),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Match {
    pub rule: Rule,
    pub terms: Vec<Term>,
}

impl Match {
    pub fn new(rule: Rule, terms: Vec<Term>) -> Self {
        Self { rule, terms }
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct MatchId(u32);

impl std::fmt::Debug for MatchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MatchId({})", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Grammar {
    /// All the matches within the grammar.
    matches: Vec<Match>,
    /// A map from a rule to all the matches that the rule has.
    rule_matches: HashMap<Rule, Vec<MatchId>>,
    /// A map from a match to its index within the rule_matches map.
    rule_match_index: HashMap<MatchId, usize>,
}

impl Grammar {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
            rule_matches: HashMap::new(),
            rule_match_index: HashMap::new(),
        }
    }

    pub fn add(&mut self, rule: Rule, terms: Vec<Term>) -> MatchId {
        let id = self.matches.len();
        self.matches.push(Match::new(rule, terms));

        let match_id = MatchId(id as u32);
        self.rule_matches
            .entry(rule)
            .or_insert_with(Vec::new)
            .push(match_id);
        self.rule_match_index
            .insert(match_id, self.rule_matches[&rule].len() - 1);

        match_id
    }

    pub fn add_match(&mut self, m: Match) -> MatchId {
        let id = self.matches.len();
        self.matches.push(m);
        MatchId(id as u32)
    }

    pub fn get(&self, id: MatchId) -> &Match {
        &self.matches[id.0 as usize]
    }

    pub fn iter_matches(&self) -> impl Iterator<Item = (MatchId, &Match)> {
        self.matches
            .iter()
            .enumerate()
            .map(|(i, m)| (MatchId(i as u32), m))
    }

    pub fn get_rule_from_match(&self, id: MatchId) -> Rule {
        self.get(id).rule
    }

    pub fn get_matches_from_rule(&self, rule: Rule) -> &[MatchId] {
        self.rule_matches
            .get(&rule)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_rule_match_index(&self, id: MatchId) -> usize {
        self.rule_match_index[&id]
    }

    pub fn iter_rules(&self) -> impl '_ + Iterator<Item = Rule> {
        self.rule_matches.keys().cloned()
    }

    pub fn root_id(&self) -> MatchId {
        MatchId(0)
    }
}
