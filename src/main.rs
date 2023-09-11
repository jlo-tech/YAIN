extern crate pest;
#[macro_use]
extern crate pest_derive;

mod parser;
mod interaction;

use crate::parser::{ast, AstNode, LangParser, Semantic};
use crate::parser::transform::compileProgram;

fn main() {

    let s = "
    ADD(x) # S(y) = ADD(S(x)) ~ y
    ADD(O) # O() = O
    ADD(x) # O() = x
    ADD(O) ~ S(O)".to_string();

    let an = ast(&s);
    println!("AST: {:?}", an.clone());

    let pg = Semantic::Program::fromAst(an.clone());
    println!("{:?}", pg);

    let mut compiled = compileProgram(pg);
    println!("{:?}", compiled.1);

    compiled.0.reduce();

    println!("{:?}", compiled.0.interaction_net.heap);
}