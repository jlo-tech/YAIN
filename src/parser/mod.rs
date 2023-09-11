pub mod transform;

use std::collections::VecDeque;
use std::os::unix::io::AsFd;
use pest::Parser;
use pest::iterators::Pair;
use crate::parser::Semantic::{AgentType, Principal, PrincipalType, Program};
use crate::parser::Semantic::AgentType::Agent;

#[derive(pest_derive::Parser)]
#[grammar = "parser/lang.pest"]
pub struct LangParser;

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Id(String),
    Var(String),
    Cons(String),
    Agent(Box<AstNode>, Vec<AstNode>),
    Principal(Box<AstNode>, Box<AstNode>),
    Equation(Box<AstNode>, Box<AstNode>, Vec<AstNode>),
    Program(Vec<AstNode>, Box<AstNode>),
}

pub fn ast(text: &String) -> AstNode {

    let parser = LangParser::parse(Rule::program, text).unwrap();

    fn value(rule: &Pair<Rule>) -> AstNode {
        return match rule.as_rule() {
            Rule::id => {
                AstNode::Id(String::from(rule.as_str()))
            }
            Rule::var => {
                AstNode::Var(String::from(rule.as_str()))
            }
            Rule::cons => {
                AstNode::Cons(String::from(rule.as_str()))
            }
            Rule::agent => {
                // Iterator
                let mut it = rule.clone().into_inner().into_iter();
                // Id is first node
                let id = value(&it.next().unwrap());
                // Get rest of nodes
                let mut v = vec![];
                for i in it {
                    v.push(value(&i));
                }
                // Return
                AstNode::Agent(Box::new(id), v)
            }
            Rule::principal => {
                // Iterator
                let mut it = rule.clone().into_inner().into_iter();
                AstNode::Principal(Box::new(value(&it.next().unwrap())),
                                   Box::new(value(&it.next().unwrap())))
            }
            Rule::equation => {
                let mut it = rule.clone().into_inner().into_iter();
                // Left side of equation
                let mut left = Box::new(value(&it.next().unwrap()));
                // Right side of equation
                let mut right = Box::new(value(&it.next().unwrap()));
                // Connections
                let mut v = vec![];
                for i in it {
                    v.push(value(&i));
                }
                AstNode::Equation(left, right, v)
            }
            Rule::program => {
                // Iterator
                let mut it = rule.clone().into_inner().into_iter();
                // Queue
                let mut vd = VecDeque::new();
                // Collect elements
                for i in it {
                    vd.push_back(value(&i));
                }
                // Last one is term that should be evaluated
                let term = vd.pop_back().unwrap();
                AstNode::Program(vd.try_into().unwrap(), Box::new(term))
            }
            _ => {
                panic!("AST generation failed")
            }
        };
    }

    return value(&parser.into_iter().next().unwrap());
}

pub mod Semantic {

    #[derive(Debug, Clone, PartialEq)]
    pub enum AgentType {
        Var,
        Cons,
        Agent,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum PrincipalType {
        Pure,
        Var,
        Cons,
    }

    #[derive(Debug, Clone)]
    pub struct Agent {
        pub name: String,
        pub atype: AgentType,
        pub ports: Vec<Agent>,
    }

    #[derive(Debug, Clone)]
    pub struct Principal {
        pub ptype: PrincipalType,
        pub left: Agent,
        pub right: Agent,
    }

    #[derive(Debug, Clone)]
    pub struct Equation {
        pub left: Agent,
        pub right: Agent,
        pub principals: Vec<Principal>,
    }

    #[derive(Debug, Clone)]
    pub struct Program {
        pub equations: Vec<Equation>,
        pub term: Principal,
    }
}

impl Semantic::Program {

    pub fn fromAst(ast: AstNode) -> Self {

        fn scanAgent(ast: AstNode) -> Semantic::Agent {
            match ast {
                AstNode::Var(s) => {
                    Semantic::Agent{
                        name: s.clone(),
                        atype: AgentType::Var,
                        ports: vec![],
                    }
                }
                AstNode::Cons(s) => {
                    Semantic::Agent{
                        name: s.clone(),
                        atype: AgentType::Cons,
                        ports: vec![],
                    }
                }
                AstNode::Agent(id, ports) => {
                    // Extract name
                    let s = match *id {
                        AstNode::Id(s) => {
                            s
                        }
                        _ => {"".to_string()}
                    };
                    // Scan sub agents
                    let mut pv = vec![];
                    for p in ports {
                        pv.push(scanAgent(p.clone()));
                    }
                    // Return
                    Semantic::Agent{
                        name: s.clone(),
                        atype: AgentType::Agent,
                        ports: pv,
                    }
                },
                _ => {
                    panic!("Illegal argument");
                }
            }
        }

        fn scanPrincipal(ast: AstNode) -> Semantic::Principal {
            match ast {
                AstNode::Principal(left, right) => {
                    Semantic::Principal{
                        ptype: PrincipalType::Pure,
                        left: scanAgent(*left),
                        right: scanAgent(*right),
                    }
                }
                AstNode::Var(s) => {
                    Semantic::Principal{
                        ptype: PrincipalType::Var,
                        left: Semantic::Agent{
                            name: s.clone(),
                            atype: AgentType::Var,
                            ports: vec![],
                        },
                        // Right is ignored in this case
                        right: Semantic::Agent{
                            name: "".to_string(),
                            atype: AgentType::Agent,
                            ports: vec![],
                        }
                    }
                }
                AstNode::Cons(s) => {
                    Semantic::Principal{
                        ptype: PrincipalType::Cons,
                        left: Semantic::Agent{
                            name: s.clone(),
                            atype: AgentType::Cons,
                            ports: vec![],
                        },
                        // Right is ignored in this case
                        right: Semantic::Agent{
                            name: "".to_string(),
                            atype: AgentType::Agent,
                            ports: vec![],
                        }
                    }
                }
                _ => {
                    panic!("Illegal argument")
                }
            }
        }

        fn scanEquation(ast: AstNode) -> Semantic::Equation {
            match ast {
                AstNode::Equation(left, right, principals) => {

                    let mut pv = vec![];
                    for p in principals {
                        pv.push(scanPrincipal(p));
                    }

                    Semantic::Equation{
                        left: scanAgent(*left),
                        right: scanAgent(*right),
                        principals: pv,
                    }
                }
                _ => {
                    panic!("Illegal argument")
                }
            }
        }

        fn scanProgram(ast: AstNode) -> Semantic::Program {
            match ast {
                AstNode::Program(equations, term) => {

                    let mut ev = vec![];
                    for e in equations {
                        ev.push(scanEquation(e));
                    }

                    return Semantic::Program{
                        equations: ev,
                        term: scanPrincipal(*term),
                    }
                },
                _ => {
                    panic!("Illegal argument")
                }
            };
        }

        return scanProgram(ast);
    }
}