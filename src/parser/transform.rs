use std::cell::RefCell;
use std::collections::HashMap;
use crate::interaction::vm::{Instruction, VM};
use crate::parser::*;

// Builds agent and leaves id on stack
fn build_agent(type_mapping: &HashMap<String, u64>,
               variable_mapping: &HashMap<String, u64>,
               agent: &Semantic::Agent) -> Vec<Instruction>
{
    let mut code = vec![];

    match agent.atype {
        AgentType::Var => {
            code.push(Instruction::PUSH(variable_mapping.get(&agent.name).unwrap().clone()));
        }
        AgentType::Cons => {
            // Id
            code.push(Instruction::GEN);
            // Save id for later
            code.push(Instruction::DUP);
            // Type
            code.push(Instruction::CONST(type_mapping.get(&agent.name).unwrap().clone()));
            // Create agent
            code.push(Instruction::NEW_AGENT);
        }
        Agent => {
            // Id
            code.push(Instruction::GEN);
            // Save id for later
            code.push(Instruction::DUP);
            // Type
            code.push(Instruction::CONST(type_mapping.get(&agent.name).unwrap().clone()));
            // Create agent
            code.push(Instruction::NEW_AGENT);
            // Connect and create children
            for port in &agent.ports {
                // Copy own id
                code.push(Instruction::DUP);
                // Create other agents
                code.append(&mut build_agent(&type_mapping, &variable_mapping, &port));
                // Connect auxiliary ports
                code.push(Instruction::CONST(0));
                code.push(Instruction::CONST(0));
                code.push(Instruction::BIND);
            }
        }
    }

    return code;
}

pub fn compileEquation(equation: &Semantic::Equation, type_mapping: &HashMap<String, u64>) -> Vec<Instruction> {

    // Code
    let mut code : Vec<Instruction> = Vec::new();
    // Map var name to postion of id in memory
    let mut variable_mapping: HashMap<String, u64> = HashMap::new();

    // 1. get agent id and port for every var on left side of equation
    // Since on the left side only depth 1 agents are allowed we can determine them easily
    // Then save var ids on heap for easy access

    // Index
    let mut index: u64 = 0;

    // Process agent
    let lports = equation.left.ports.clone();
    // Skip principal port
    for port in 0..lports.len() {
        if lports[port].atype == AgentType::Var {
            if !(variable_mapping.contains_key(&lports[port].name.clone())) {
                // Save variable mapping
                let heap_pos = variable_mapping.len() as u64 + 1;
                variable_mapping.insert(lports[port].name.clone(), heap_pos);
                // Duplicate agent id on stack
                code.push(Instruction::DUP);
                // Push port index (but skip principal port)
                code.push(Instruction::CONST(index + 1));
                // Get id of connected agent
                code.push(Instruction::PORT);
                // Store id on heap
                code.push(Instruction::POP(heap_pos));
            }
        }
        // Increment index
        index = index + 1;
    }

    // Store id of second agent on heap position 0
    code.push(Instruction::POP(0));

    // Reset index
    index = 0;

    // Process agent
    let rports = equation.right.ports.clone();
    // Skip principal port
    for port in 0..rports.len() {
        if rports[port].atype == AgentType::Var {
            if !(variable_mapping.contains_key(&rports[port].name.clone())) {
                // Save variable mapping
                let heap_pos = variable_mapping.len() as u64 + 1;
                variable_mapping.insert(rports[port].name.clone(), heap_pos);
                // Duplicate agent id on stack
                code.push(Instruction::DUP);
                // Push port index (but skip principal port)
                code.push(Instruction::CONST(index + 1));
                // Get id of connected agent
                code.push(Instruction::PORT);
                // Store id on heap
                code.push(Instruction::POP(heap_pos));
            }
        }
        index = index + 1;
    }

    // Restore id of second agent from heap
    code.push(Instruction::PUSH(0));

    // 2. Delete (old) agents on left side
    code.push(Instruction::DROP_AGENT);
    code.push(Instruction::DROP_AGENT);

    // 3. build up new graph and connect vars appropriately

    // Create new connections
    for principal in &equation.principals {
        if principal.ptype == PrincipalType::Pure {
            // Build agents and create auxiliary connections
            code.append(&mut build_agent(&type_mapping, &variable_mapping, &principal.left));
            code.append(&mut build_agent(&type_mapping, &variable_mapping, &principal.right));

            // Bind agents by creating principal connection
            code.push(Instruction::CONST(1));
            code.push(Instruction::CONST(1));
            code.push(Instruction::BIND);
        }
        else if principal.ptype == PrincipalType::Var {
            code.append(&mut build_agent(&type_mapping, &variable_mapping, &principal.left));
        }
        else if principal.ptype == PrincipalType::Cons {
            code.append(&mut build_agent(&type_mapping, &variable_mapping, &principal.left));
        }
    }

    return code;
}

pub fn compileProgram(program: Semantic::Program) -> (VM, HashMap<String, u64>) {

    // VM
    let mut vm = VM::new();
    // Code
    let mut code: Vec<Instruction> = Vec::new();

    // Create type mapping
    let mut type_mapping: RefCell<HashMap<String, u64>> = RefCell::new(HashMap::new());

    fn traverse_types(agent: &Semantic::Agent, mut type_mapping: &RefCell<HashMap<String, u64>>) {

        let mut handle = type_mapping.borrow_mut();

        // Traverse agent itself
        if agent.atype != AgentType::Var {
            if !(handle.contains_key(&agent.name)) {
                let len = handle.len() as u64;
                handle.insert(agent.name.clone(), len);
            }
        }

        // Release mutable borrow
        drop(handle);

        // Traverse ports
        for port in &agent.ports {
            traverse_types(&port, &type_mapping);
        }
    }

    for equation in &program.equations {
        // Traverse left and right side of equation
        traverse_types(&equation.left, &type_mapping);
        traverse_types(&equation.right, &type_mapping);
        // Traverse principals
        for principal in &equation.principals {
            traverse_types(&principal.left, &type_mapping);
            traverse_types(&principal.right, &type_mapping);
        }
    }

    // Generate code for every equation
    for equation in program.equations {
        let rule_types = (
            *type_mapping.clone().into_inner().get(&equation.left.name.clone()).unwrap(),
            *type_mapping.clone().into_inner().get(&equation.right.name.clone()).unwrap());
        let rule_code = compileEquation(&equation, &type_mapping.clone().into_inner());
        // Save in vm
        vm.rules.insert(rule_types, rule_code);
    }

    // Generate code for principal

    // Construct agents
    fn build_term(type_mapping: &HashMap<String, u64>, agent: &Semantic::Agent) -> Vec<Instruction> {
        // Code
        let mut code = vec![];
        // Process agent
        match agent.atype {
            AgentType::Var => {
                panic!("Vars not allowed here");
            }
            AgentType::Cons => {
                // Id
                code.push(Instruction::GEN);
                // Save id for later
                code.push(Instruction::DUP);
                // Type
                code.push(Instruction::CONST(type_mapping.get(&agent.name).unwrap().clone()));
                // Create agent
                code.push(Instruction::NEW_AGENT);
            }
            AgentType::Agent => {
                // Id
                code.push(Instruction::GEN);
                // Save id for later
                code.push(Instruction::DUP);
                // Type
                code.push(Instruction::CONST(type_mapping.get(&agent.name).unwrap().clone()));
                // Create agent
                code.push(Instruction::NEW_AGENT);
                // Connect and create children
                for port in &agent.ports {
                    // Copy own id
                    code.push(Instruction::DUP);
                    // Create other agents
                    code.append(&mut build_term(&type_mapping, &port));
                    // Connect auxiliary ports
                    code.push(Instruction::CONST(0));
                    code.push(Instruction::CONST(0));
                    code.push(Instruction::BIND);
                }
            }
        }

        return code;
    }
    // Create principal connection
    code.append(&mut build_term(&type_mapping.clone().into_inner(), &program.term.left));
    code.append(&mut build_term(&type_mapping.clone().into_inner(), &program.term.right));
    code.push(Instruction::CONST(1));
    code.push(Instruction::CONST(1));
    code.push(Instruction::BIND);
    // Load code into vm
    vm.code = code;
    // Run initial code to create principal connection
    vm.run();

    // Return finally prepared vm
    return (vm, type_mapping.into_inner());
}