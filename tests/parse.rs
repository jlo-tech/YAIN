#[path = "../src/interaction/mod.rs"]
mod interaction;

#[path = "../src/parser/mod.rs"]
mod parser;

#[cfg(test)]
mod unit_tests {

    use crate::parser::{ast, AstNode, Semantic};
    use crate::parser::Semantic::PrincipalType;
    use crate::parser::transform::compileProgram;

    #[test]
    pub fn test_ast() {
        let s = "
        INC(x) # S(y) = S() ~ S(y)
        INC(x) # O() = x
        INC(O) ~ S(O)".to_string();

        let an = ast(&s);

        assert_eq!(an,
         AstNode::Program(
             vec![
                AstNode::Equation(
                    Box::new(AstNode::Agent(
                                Box::new(AstNode::Id("INC".to_string())),
                                vec![AstNode::Var("x".to_string())])),
                    Box::new(AstNode::Agent(
                                Box::new(AstNode::Id("S".to_string())),
                                vec![AstNode::Var("y".to_string())])),
                    vec![
                        AstNode::Principal(
                            Box::new(AstNode::Agent(
                                Box::new(AstNode::Id("S".to_string())),
                                vec![])),
                            Box::new(AstNode::Agent(
                                Box::new(AstNode::Id("S".to_string())),
                                vec![AstNode::Var("y".to_string())])))]),

                AstNode::Equation(
                    Box::new(AstNode::Agent(
                        Box::new(AstNode::Id("INC".to_string())),
                        vec![AstNode::Var("x".to_string())])),
                    Box::new(AstNode::Agent(Box::new(AstNode::Id("O".to_string())), vec![])),
                    vec![AstNode::Var("x".to_string())])
             ],
             Box::new(AstNode::Principal(
                 Box::new(
                                AstNode::Agent(Box::new(AstNode::Id("INC".to_string())),
                                               vec![AstNode::Cons("O".to_string())])),
                 Box::new(
                                AstNode::Agent(Box::new(AstNode::Id("S".to_string())),
                                               vec![AstNode::Cons("O".to_string())])))))
        );
    }

    #[test]
    pub fn test_semantic() {

        let s = "
        INC(x) # S(y) = S() ~ S(y)
        INC(x) # O() = x
        INC(O) ~ S(O)".to_string();

        let an = ast(&s);

        let pg = Semantic::Program::fromAst(an.clone());

        assert_eq!(pg.equations.len(), 2);
        assert_eq!(pg.equations[0].left.name, "INC".to_string());
        assert_eq!(pg.equations[0].right.name, "S".to_string());

        assert_eq!(pg.equations[0].left.ports.len(), 1);
        assert_eq!(pg.equations[0].right.ports.len(), 1);
        assert_eq!(pg.equations[1].principals.len(), 1);
        assert_eq!(pg.equations[1].principals[0].ptype, PrincipalType::Var);
    }

    #[test]
    pub fn test_compilation() {

        let s = "
        INC(x) # S(y) = S() ~ S(y)
        INC(x) # O() = x
        INC(O) ~ S(O)".to_string();

        let an = ast(&s);
        let pg = Semantic::Program::fromAst(an.clone());
        let tup = compileProgram(pg);
        let mut vm = tup.0;
        let tm = tup.1;

        assert_eq!(*tm.get("INC").unwrap(), 0);
        assert_eq!(*tm.get("S").unwrap(), 1);
        assert_eq!(*tm.get("O").unwrap(), 2);

        vm.reduce();

        assert_eq!(vm.interaction_net.heap.len(), 4);
        assert_eq!(vm.rules.len(), 2);
    }
}