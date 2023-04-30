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
        vec![
            Term::Token(Token::Start),
            Term::Rule(Rule::Expr),
            Term::Token(Token::Eof),
        ],
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

    grammar.add(Rule::Term, vec![Term::Group(Group::Parens, Rule::S)]);
    grammar.add(
        Rule::Term,
        vec![
            Term::Token(Token::LParen),
            Term::Rule(Rule::Expr),
            Term::Token(Token::RParen),
        ],
    );

    grammar
}

fn make_calc2_grammar() -> Grammar {
    let mut grammar = Grammar::new();
    grammar.add(
        Rule::S,
        vec![
            Term::Token(Token::Start),
            Term::Rule(Rule::Expr),
            Term::Token(Token::Eof),
        ],
    );
    grammar.add(Rule::Expr, vec![Term::Rule(Rule::Add)]);
    grammar.add(
        Rule::Add,
        vec![
            Term::Rule(Rule::Add),
            Term::Rule(Rule::Op2),
            Term::Rule(Rule::Mul),
        ],
    );
    grammar.add(Rule::Add, vec![Term::Rule(Rule::Mul)]);
    grammar.add(
        Rule::Mul,
        vec![
            Term::Rule(Rule::Mul),
            Term::Rule(Rule::Op1),
            Term::Rule(Rule::Term),
        ],
    );
    grammar.add(Rule::Mul, vec![Term::Rule(Rule::Term)]);
    grammar.add(Rule::Term, vec![Term::Token(Token::Num)]);

    grammar.add(Rule::Op1, vec![Term::Token(Token::Star)]);
    grammar.add(Rule::Op1, vec![Term::Token(Token::Slash)]);

    grammar.add(Rule::Op2, vec![Term::Token(Token::Plus)]);
    grammar.add(Rule::Op2, vec![Term::Token(Token::Minus)]);

    grammar.add(Rule::Term, vec![Term::Group(Group::Parens, Rule::S)]);
    grammar.add(
        Rule::Term,
        vec![
            Term::Token(Token::LParen),
            Term::Rule(Rule::Expr),
            Term::Token(Token::RParen),
        ],
    );

    grammar
}

fn make_struct_fn_grammar() -> Grammar {
    let mut grammar = Grammar::new();
    grammar.add(
        Rule::S,
        vec![
            Term::Token(Token::Start),
            Term::Rule(Rule::Expr),
            Term::Token(Token::Eof),
        ],
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
        vec![
            Term::Token(Token::Start),
            Term::Rule(Rule::Expr),
            Term::Token(Token::Eof),
        ],
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
    let grammar = make_struct_fn_grammar();

    let solver = solver::GrammarSolver::new(grammar);

    // let tokens = vec![
    //     ITokenOrGroup::Token(Token::Start),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Plus),
    //     ITokenOrGroup::Token(Token::Plus),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Star),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Plus),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Eof),
    // ];

    // let tokens = vec![
    //     ITokenOrGroup::Token(Token::Start),
    //     ITokenOrGroup::Group(vec![
    //         ITokenOrGroup::Token(Token::Start),
    //         ITokenOrGroup::Token(Token::Num),
    //         ITokenOrGroup::Token(Token::Plus),
    //         ITokenOrGroup::Token(Token::Num),
    //         ITokenOrGroup::Token(Token::Eof),
    //     ]),
    //     ITokenOrGroup::Token(Token::Star),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Plus),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Eof),
    // ];

    let tokens = vec![
        ITokenOrGroup::Token(Token::Start),
        // ITokenOrGroup::Token(Token::Pub),
        // ITokenOrGroup::Token(Token::Star),
        ITokenOrGroup::Token(Token::Struct),
        ITokenOrGroup::Token(Token::Eof),
    ];

    // let tokens = vec![
    //     ITokenOrGroup::Token(Token::Start),
    //     // ITokenOrGroup::Token(Token::Num),
    //     // ITokenOrGroup::Token(Token::Num),
    //     // ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Num),
    //     // ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Num),
    //     ITokenOrGroup::Token(Token::Eof),
    // ];

    let result = solve(&solver, tokens);
    println!("{}", result);

    // dbg!(&grammar);
}
