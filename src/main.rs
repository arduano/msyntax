use interpreter::{solve, ITokenOrGroup};

use crate::matches::*;

mod analysis;
mod interpreter;
mod matches;
mod ref_list;
mod solver;
mod structures;

fn make_calc_grammar() -> Grammar {
    let mut grammar = Grammar::new();
    grammar.add(
        Rule::S,
        vec![Term::Rule(Rule::Expr), Term::Token(Token::Eof)],
    );
    grammar.add(Rule::Expr, vec![Term::Rule(Rule::Add)]);
    grammar.add(
        Rule::Add,
        vec![
            Term::Rule(Rule::Add),
            Term::Token(Token::Plus),
            Term::Rule(Rule::Mul),
        ],
    );
    grammar.add(Rule::Add, vec![Term::Rule(Rule::Mul)]);
    grammar.add(
        Rule::Mul,
        vec![
            Term::Rule(Rule::Mul),
            Term::Token(Token::Star),
            Term::Rule(Rule::Term),
        ],
    );
    grammar.add(Rule::Mul, vec![Term::Rule(Rule::Term)]);
    grammar.add(Rule::Term, vec![Term::Token(Token::Num)]);
    grammar.add(Rule::Term, vec![Term::Group(Group::Parens, Rule::Expr)]);

    grammar
}

fn make_struct_fn_grammar() -> Grammar {
    let mut grammar = Grammar::new();
    grammar.add(
        Rule::S,
        vec![Term::Rule(Rule::Expr), Term::Token(Token::Eof)],
    );
    grammar.add(Rule::Expr, vec![Term::Rule(Rule::Struct)]);
    grammar.add(Rule::Expr, vec![Term::Rule(Rule::Fn)]);
    grammar.add(
        Rule::Struct,
        vec![Term::Rule(Rule::Vis), Term::Token(Token::Struct)],
    );
    grammar.add(
        Rule::Fn,
        vec![Term::Rule(Rule::Vis), Term::Token(Token::Fn)],
    );
    grammar.add(Rule::Vis, vec![]);
    grammar.add(
        Rule::Vis,
        vec![Term::Token(Token::Pub), Term::Rule(Rule::VisModifier)],
    );
    grammar.add(Rule::VisModifier, vec![Term::Token(Token::Star)]);
    grammar.add(Rule::VisModifier, vec![]);

    grammar
}

fn make_array_grammar() -> Grammar {
    let mut grammar = Grammar::new();
    grammar.add(
        Rule::S,
        vec![Term::Rule(Rule::Expr), Term::Token(Token::Eof)],
    );
    grammar.add(
        Rule::Expr,
        vec![Term::Rule(Rule::Expr), Term::Rule(Rule::Term)],
    );
    grammar.add(Rule::Expr, vec![Term::Rule(Rule::Term)]);
    grammar.add(Rule::Expr, vec![]);
    grammar.add(Rule::Term, vec![Term::Token(Token::Num)]);

    grammar
}

fn main() {
    let grammar = make_array_grammar();

    let solver = solver::GrammarSolver::new(grammar).unwrap();

    // let tokens = vec![
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Plus),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Star),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Plus),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Eof),
    // ];

    let tokens = vec![
        // ITokenOrGroup::Token(Token::Pub),
        // ITokenOrGroup::Token(Token::Star),
        ITokenOrGroup::Token(Token::Struct),
        ITokenOrGroup::Token(Token::Eof),
    ];

    let tokens = vec![
        // ITokenOrGroup::Token(Token::Num),
        // ITokenOrGroup::Token(Token::Num),
        // ITokenOrGroup::Token(Token::Num),
        ITokenOrGroup::Token(Token::Num),
        // ITokenOrGroup::Token(Token::Num),
        ITokenOrGroup::Token(Token::Num),
        ITokenOrGroup::Token(Token::Eof),
    ];

    let result = solve(&solver, tokens);
    dbg!(result);

    // dbg!(&grammar);
}
