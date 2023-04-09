// struct Function = {
//     viz: <Viz>;
//     Fn: match;
//     name: <Ident>;
//     "("
//     args: <FnArg*>
//     ")"
// };

// struct FnArg = {
//     name: <Ident>;
//     <Colon>;
//     ty: <Type>;
// };

f!{

type Num = r#"\d+"#;

enum Term {
    Num(Num),
    Group( "(" Expr ")" ),
    BinOp,
}

enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

match LowOp: Op {
    "+" => Op::Add,
    "-" => Op::Sub,
}

match HighOp: Op {
    "*" => Op::Mul,
    "/" => Op::Div,
}

match LowOpMap: BinOp {
    "+" => Op::Add,
    "-" => Op::Sub,
    "*" => Op::Mul,
    "/" => Op::Div,
}

struct BinOp {
    left: Term,
    op: Op,
    right: Term,
}

type Expr: Op {
    <l:Expr> "+" <r:Factor> => l + r,
    <l:Expr> "-" <r:Factor> => l - r,
    Factor,
}






S -> A
A -> A + B | B
B -> B * C | C
C -> (A) | x

// Common stack groups:
// 1. Num(x)
// 2. Paren()
// 3. Paren(A)
// 4. Plus(A)
// 5. Plus(A, B)
// 6. Mul(B)
// 7. Mul(B, C)

// For the input "1 + 2 * 3", the following would happen:

// Stack: []
// Token: 1
//
// Stack: [Num(1)]
// Token: +
//
// Stack: [Plus(Num(1))]
// Token: 2
//
// Stack: [Plus(Num(1)), Num(2)]
// Token: *
//
// Stack: [Plus(Num(1)), Mul(Num(2))]
// Token: 3
//
// Stack: [Plus(Num(1)), Mul(Num(2), Num(3))]
// Token: EOF
//
// Stack: [Plus(Num(1), Mul(Num(2), Num(3)))]

// For the input "1 * 2 + 3", the following would happen:

// Stack: []
// Token: 1
//
// Stack: [Num(1)]
// Token: *
//
// Stack: [Mul(Num(1))]
// Token: 2
//
// Stack: [Mul(Num(1), Num(2))]
// Token: +
//
// Stack: [Mul(Num(1), Num(2)), Num(3)]
// Token: EOF
//
// Stack: [Plus(Mul(Num(1), Num(2)), Num(3))]

}

