use core::panic;

use crate::{
    matches::{MatchId, Rule, Token},
    solver::{
        EmptySolverRuleValue, EmptyWrapAction, FirstSet, FollowSet, GrammarSolver, MatchIndex,
        TokenOrGroup,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ITokenOrGroup {
    Token(Token),
    Group(Vec<ITokenOrGroup>),
}

pub struct TokenReader<'a> {
    pub tokens: &'a [ITokenOrGroup],
    pub index: usize,
}

impl<'a> TokenReader<'a> {
    pub fn new(tokens: &'a [ITokenOrGroup]) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn does_match(&self, by: usize, token2: &TokenOrGroup) -> bool {
        let token = self.tokens.get(self.index + by);

        let Some(token) = token else {
            return false;
        };

        match token {
            ITokenOrGroup::Token(token) => match token2 {
                TokenOrGroup::Token(token2) => token == token2,
                TokenOrGroup::Group(_, _) => false,
            },
            ITokenOrGroup::Group(_) => match token2 {
                TokenOrGroup::Token(_) => false,
                TokenOrGroup::Group(_, _) => true,
            },
        }
    }

    pub fn next(&mut self) -> Option<&ITokenOrGroup> {
        let token = self.tokens.get(self.index);
        self.index += 1;
        token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Token(Token),
    Rule(RuleValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchValue {
    pub match_id: MatchId,
    pub values: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleValue {
    pub rule: Rule,
    pub match_id: MatchId,
    pub values: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackItem {
    linked_to_above: bool,
    match_value: MatchValue,
}

pub struct Interpreter<'a> {
    stack: Vec<StackItem>,
    token_reader: TokenReader<'a>,
    solver: &'a GrammarSolver,
}

enum WrapStatusAction<'a> {
    /// Seal the current match, then wrap it using wrap_above, then insert a new match onto the stack with this as a child.
    WrapWith {
        wrap_above: &'a [EmptyWrapAction],
        match_id: MatchId,

        /// Append these into the newly created parent before adding the child
        append_before: &'a [EmptySolverRuleValue],

        /// Append these before sealing
        seal_append: &'a [EmptySolverRuleValue],
    },
    /// Seal the current match, then wrap it in the wrap_above, until finally inserting it into the next parent.
    InsertIntoAbove {
        wrap_above: &'a [EmptyWrapAction],

        /// Append these before sealing
        seal_append: &'a [EmptySolverRuleValue],
    },
}

enum WrapStatus<'a> {
    Action(WrapStatusAction<'a>),
    Matches,
    Error,
}

enum HasChild {
    Yes,
    No,
}

pub fn solve(solver: &GrammarSolver, tokens: Vec<ITokenOrGroup>) -> RuleValue {
    let interpreter = Interpreter {
        stack: Vec::new(),
        token_reader: TokenReader::new(&tokens),
        solver: &solver,
    };

    interpreter.solve(solver.root_rule())
}

impl<'a> Interpreter<'a> {
    fn solve(mut self, root_rule: Rule) -> RuleValue {
        let first_set = self.solver.first_set_for_rule(root_rule);
        self.solve_first_set(&[], first_set);

        loop {
            dbg!(&self.stack);
            let mi = self.get_match_index_of_top_stack_item();

            let follow_set = self.solver.follow_set_for_match(mi);
            if self.solve_follow_sets(follow_set) {
                continue;
            }

            let mut reduce_stack = Vec::new();

            let mut i = self.stack.len() - 1;
            loop {
                let has_child = if i != self.stack.len() - 1 {
                    HasChild::Yes
                } else {
                    HasChild::No
                };

                let status = self.get_wrap_status_for_stack_item(i, has_child);

                match status {
                    WrapStatus::Matches => break,
                    WrapStatus::Error => panic!("No follow set matched, and no rule to reduce to"),
                    WrapStatus::Action(action) => {
                        let should_break = match action {
                            WrapStatusAction::WrapWith { .. } => true,
                            WrapStatusAction::InsertIntoAbove { .. } => false,
                        };
                        reduce_stack.push(action);

                        if should_break {
                            break;
                        }
                    }
                }

                if i == 0 {
                    break;
                }

                i -= 1;
            }

            for action in reduce_stack {
                match action {
                    WrapStatusAction::WrapWith {
                        wrap_above,
                        match_id,
                        append_before,
                        seal_append,
                    } => {
                        println!("WrapWith");

                        self.append_emptys(seal_append);

                        for wrap in wrap_above {
                            self.wrap_top_stack_item_into_empty(wrap);
                        }

                        let value = self.seal_top_stack_item();

                        let match_value = MatchValue {
                            match_id,
                            values: vec![],
                        };
                        let stack_value = StackItem {
                            linked_to_above: false,
                            match_value,
                        };
                        self.stack.push(stack_value);

                        self.append_emptys(append_before);
                        self.append_value(value);
                    }
                    WrapStatusAction::InsertIntoAbove {
                        wrap_above,
                        seal_append,
                    } => {
                        println!("InsertIntoAbove");

                        self.append_emptys(seal_append);

                        for wrap in wrap_above {
                            self.wrap_top_stack_item_into_empty(wrap);
                        }

                        let value = self.seal_top_stack_item();

                        if self.stack.len() == 0 {
                            return match value {
                                Value::Rule(rule) => rule,
                                _ => panic!("Expected rule value"),
                            };
                        }

                        self.append_value(value);
                    }
                }
            }
        }
    }

    fn get_matching_first_set<'b>(&self, first_sets: &'b [FirstSet]) -> Option<&'b FirstSet> {
        first_sets
            .iter()
            .find(|first_set| self.matches_tokens(&first_set.tokens))
    }

    fn insert_first_set_data(&mut self, set: &FirstSet) {
        for action in &set.then {
            let match_value = MatchValue {
                match_id: action.id,
                values: Self::process_empty_items(&action.append_empty_fields),
            };

            let stack_value = StackItem {
                linked_to_above: action.linked_to_above,
                match_value,
            };

            self.stack.push(stack_value);
        }
    }

    fn solve_first_set(
        &mut self,
        append_emptys: &[EmptySolverRuleValue],
        first_sets: &[FirstSet],
    ) -> bool {
        if let Some(set) = self.get_matching_first_set(first_sets) {
            if append_emptys.len() > 0 {
                self.append_emptys(append_emptys);
            }

            self.insert_first_set_data(set);
            // self.parse_tokens(&set.tokens); //FIXME: uncomment?
            true
        } else {
            false
        }
    }

    fn does_follow_set_match(&self, follow_sets: &[FollowSet]) -> bool {
        for set in follow_sets {
            match set {
                FollowSet::Direct(direct) => {
                    if self.matches_tokens(&direct.tokens) {
                        return true;
                    }
                }
                FollowSet::Enter(enter) => {
                    let first_sets = self.solver.first_set_for_rule(enter.rule);
                    if self.get_matching_first_set(first_sets).is_some() {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn solve_follow_sets(&mut self, follow_sets: &[FollowSet]) -> bool {
        for set in follow_sets {
            match set {
                FollowSet::Direct(direct) => {
                    if self.matches_tokens(&direct.tokens) {
                        self.append_emptys(&direct.append_extra_emptys);
                        self.parse_tokens(&direct.tokens);
                        return true;
                    }
                }
                FollowSet::Enter(enter) => {
                    let first_sets = self.solver.first_set_for_rule(enter.rule);
                    if self.solve_first_set(&enter.append_extra, first_sets) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn parse_tokens(&mut self, tokens: &[TokenOrGroup]) {
        let top_value = self.stack.last_mut().unwrap();

        for token in tokens {
            match token {
                TokenOrGroup::Token(_) => {
                    let next_item = self.token_reader.next().unwrap();
                    let next_token = match next_item {
                        ITokenOrGroup::Token(token) => token,
                        ITokenOrGroup::Group(_) => panic!("Expected token, got group"),
                    };

                    let value = Value::Token(next_token.clone());
                    top_value.match_value.values.push(value);
                }
                TokenOrGroup::Group(_, rule) => {
                    let next_item = self.token_reader.next().unwrap();
                    let next_token_reader = match next_item {
                        ITokenOrGroup::Token(_) => panic!("Expected group, got token"),
                        ITokenOrGroup::Group(tokens) => TokenReader::new(tokens.as_slice()),
                    };

                    let interpreter = Interpreter {
                        stack: Vec::new(),
                        token_reader: next_token_reader,
                        solver: self.solver,
                    };

                    let rule_value = interpreter.solve(*rule);

                    let value = Value::Rule(rule_value);
                    top_value.match_value.values.push(value);
                }
            }
        }
    }

    fn append_emptys(&mut self, tokens: &[EmptySolverRuleValue]) {
        let top_value = self.stack.last_mut().unwrap();

        for token in tokens {
            let value = Self::process_empty_item(token);
            top_value.match_value.values.push(value);
        }
    }

    fn append_value(&mut self, value: Value) {
        let top_value = self.stack.last_mut().unwrap();
        top_value.match_value.values.push(value);
    }

    fn process_empty_item(item: &EmptySolverRuleValue) -> Value {
        let rule = RuleValue {
            rule: item.rule,
            match_id: item.match_value.id,
            values: Self::process_empty_items(&item.match_value.fields),
        };

        Value::Rule(rule)
    }

    fn process_empty_items(items: &[EmptySolverRuleValue]) -> Vec<Value> {
        items
            .iter()
            .map(|item| Self::process_empty_item(item))
            .collect()
    }

    fn matches_tokens(&self, tokens: &[TokenOrGroup]) -> bool {
        for (i, token) in tokens.iter().enumerate() {
            if !self.token_reader.does_match(i, token) {
                return false;
            }
        }

        true
    }

    fn seal_top_stack_item(&mut self) -> Value {
        let mi = self.get_match_index_of_top_stack_item();

        let action = self
            .solver
            .get_seal_action_for_match(mi)
            .expect("No seal action found");

        self.append_emptys(&action.append_extra);

        let stack_item = self.stack.pop().unwrap();

        let should_propagate_inner_rule = if stack_item.match_value.values.len() == 1 {
            match stack_item.match_value.values.first().unwrap() {
                Value::Token(_token) => false,
                Value::Rule(_rule) => true,
            }
        } else {
            false
        };

        let rule = if should_propagate_inner_rule {
            // This condition helps make the tree of rules look cleaner when printed
            let mut stack_item = stack_item;
            let inner_rule = match stack_item.match_value.values.pop().unwrap() {
                Value::Token(_) => unreachable!(),
                Value::Rule(rule) => rule,
            };

            RuleValue {
                rule: action.into_rule,
                match_id: inner_rule.match_id,
                values: inner_rule.values,
            }
        } else {
            RuleValue {
                rule: action.into_rule,
                match_id: stack_item.match_value.match_id,
                values: stack_item
                    .match_value
                    .values
                    .into_iter()
                    .map(|value| match value {
                        Value::Token(token) => Value::Token(token),
                        Value::Rule(rule) => Value::Rule(rule),
                    })
                    .collect(),
            }
        };

        Value::Rule(rule)
    }

    fn wrap_top_stack_item_into_empty(&mut self, empty: &EmptyWrapAction) {
        let sealed = self.seal_top_stack_item();

        let mut new_match = MatchValue {
            match_id: empty.match_id,
            values: vec![],
        };

        for left in &empty.left_empty {
            let value = Self::process_empty_item(left);
            new_match.values.push(value);
        }

        new_match.values.push(sealed);

        for right in &empty.right_empty {
            let value = Self::process_empty_item(right);
            new_match.values.push(value);
        }

        self.stack.push(StackItem {
            linked_to_above: false,
            match_value: new_match,
        });
    }

    fn get_match_index_of_stack_item(&self, index: usize) -> MatchIndex {
        let stack_item = &self.stack[index];
        MatchIndex {
            id: stack_item.match_value.match_id,
            index: stack_item.match_value.values.len(),
        }
    }

    fn get_match_index_of_top_stack_item(&self) -> MatchIndex {
        let stack_item = self.stack.last().unwrap();
        MatchIndex {
            id: stack_item.match_value.match_id,
            index: stack_item.match_value.values.len(),
        }
    }

    fn get_match_index_of_stack_item_if_child_inserted(&self, index: usize) -> MatchIndex {
        let stack_item = &self.stack[index];
        MatchIndex {
            id: stack_item.match_value.match_id,
            index: stack_item.match_value.values.len() + 1,
        }
    }

    fn get_expecting_rule_for_stack_item(&self, index: usize) -> Rule {
        let mi = self.get_match_index_of_stack_item(index);

        let match_ = self.solver.get_match(mi.id);
        let term = match_.terms[mi.index];

        *term.as_rule().expect("Expected rule")
    }

    fn get_wrap_status_for_stack_item(&self, index: usize, has_child: HasChild) -> WrapStatus<'a> {
        let mi = match has_child {
            HasChild::Yes => self.get_match_index_of_stack_item_if_child_inserted(index),
            HasChild::No => self.get_match_index_of_stack_item(index),
        };

        let follow_set = self.solver.follow_set_for_match(mi);
        if self.does_follow_set_match(follow_set) {
            return WrapStatus::Matches;
        }

        let seal_action = self.solver.get_seal_action_for_match(mi);
        let Some(seal_action) = seal_action else {
            return WrapStatus::Error;
        };

        let stack_item = &self.stack[index];

        if stack_item.linked_to_above {
            return WrapStatus::Action(WrapStatusAction::InsertIntoAbove {
                wrap_above: &[],
                seal_append: &seal_action.append_extra,
            });
        }

        // We would normally be checking above here, so if the index is 0
        // then something is wrong.
        if index == 0 {
            return WrapStatus::Error;
        }

        let parent_rule = self.get_expecting_rule_for_stack_item(index - 1);
        let child_rule = self.solver.get_match_rule(mi.id);
        let wrap_data = self
            .solver
            .get_wrap_data(parent_rule, child_rule)
            .expect("No wrap data found");

        for action in &wrap_data.wrap_actions {
            let follow_set = self.solver.follow_set_for_match(action.if_matches);
            if self.does_follow_set_match(follow_set) {
                return WrapStatus::Action(WrapStatusAction::WrapWith {
                    wrap_above: &action.wrap_actions,
                    append_before: &action.append_empty,
                    match_id: action.if_matches.id,
                    seal_append: &seal_action.append_extra,
                });
            }
        }

        if let Some(insert) = &wrap_data.insert_action {
            let parent_mi = self.get_match_index_of_stack_item_if_child_inserted(index - 1);
            let parent_follow_set = self.solver.follow_set_for_match(parent_mi);
            if self.does_follow_set_match(parent_follow_set) || parent_follow_set.len() == 0 {
                return WrapStatus::Action(WrapStatusAction::InsertIntoAbove {
                    wrap_above: &insert.wrap_actions,
                    seal_append: &seal_action.append_extra,
                });
            }
        }

        WrapStatus::Error
    }
}