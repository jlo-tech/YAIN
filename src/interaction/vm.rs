use std::ops::Index;
use std::borrow::Borrow;
use std::collections::HashMap;
use crate::interaction::*;

pub const SCRATCHPAD_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub enum Instruction {
    NOP, // No operation
    GEN, // Pushes newly generated id on stack
    CONST(u64), // Pushes constant to stack
    DUP, // Duplicate stack top
    PUSH(u64), // Load from static memory address
    POP(u64), // Store from static memory address
    NEW_AGENT, // Creates new agent on heap
    DROP_AGENT, // Deletes agent from heap
    BIND, // Connects two agents
    UNBIND, // Removes connection between agents
    PORT, // Fetches agent id and port id from stack and pushes agents[aid].ports[pid] on stack
}

#[derive(Debug)]
pub struct VM {
    pub pc: usize,
    pub code: Vec<Instruction>,
    pub stack: Vec<u64>, // Stack does only carry agent ids
    pub scratchpad: [u64; SCRATCHPAD_SIZE],
    pub interaction_net: InteractionNet,
    pub rules: HashMap<(u64, u64), Vec<Instruction>>,
}

impl VM {

    pub fn new() -> Self {
        VM {
            pc: 0,
            code: Vec::new(),
            stack: Vec::new(),
            scratchpad: [0; SCRATCHPAD_SIZE],
            interaction_net: InteractionNet::new(),
            rules: HashMap::new(),
        }
    }

    // Execute single instruction
    pub fn step(&mut self) {
        match self.code.get(self.pc).unwrap().clone() {
            Instruction::NOP => {
                // Do nothing
            }
            Instruction::GEN => {
                self.stack.push(self.interaction_net.gen_id());
            }
            Instruction::CONST(c) => {
                self.stack.push(c);
            }
            Instruction::DUP => {
                self.stack.push(self.stack.last().unwrap().clone());
            }
            Instruction::PUSH(addr) => {
                self.stack.push(self.scratchpad[addr as usize]);
            }
            Instruction::POP(addr) => {
                self.scratchpad[addr as usize] = self.stack.pop().unwrap().clone();
            }
            Instruction::NEW_AGENT => {
                let atype = self.stack.pop().unwrap().clone();
                let id = self.stack.pop().unwrap().clone();
                self.interaction_net.new_agent(id, atype);
            }
            Instruction::DROP_AGENT => {
                let id = self.stack.pop().unwrap().clone();
                self.interaction_net.drop_agent(id);
            }
            Instruction::BIND => {
                let prin1 = self.stack.pop().unwrap().clone();
                let prin0 = self.stack.pop().unwrap().clone();
                let aid1 = self.stack.pop().unwrap().clone();
                let aid0 = self.stack.pop().unwrap().clone();
                self.interaction_net.bind_agents((prin0 > 0, prin1 > 0), aid0, aid1);
            }
            Instruction::UNBIND => {
                let aid1 = self.stack.pop().unwrap().clone();
                let aid0 = self.stack.pop().unwrap().clone();
                self.interaction_net.unbind_agents(aid0, aid1);
            },
            Instruction::PORT => {
                let pid = self.stack.pop().unwrap().clone();
                let aid = self.stack.pop().unwrap().clone();
                self.stack.push(self.interaction_net.query_agent(aid).ports[pid as usize]);
            }
        }
        // Increment pc
        self.pc = self.pc + 1;
    }

    // Execute whole program
    pub fn run(&mut self) {
        while self.pc < self.code.len() {
            self.step();
        }
    }

    // Adds new rewriting rule
    pub fn new_rewrite(&mut self, atypes: (u64, u64), instructions: Vec<Instruction>) {
        self.rules.insert(atypes, instructions);
    }

    // Removes rewriting rule
    pub fn drop_rewrite(&mut self, atypes: (u64, u64)) {
        self.rules.remove(&atypes);
    }

    // Reduce interaction net
    pub fn reduce(&mut self) {
        while self.interaction_net.active_pairs.len() > 0 {
            // Fetch active pair
            let pair = self.interaction_net.active_pairs.pop().unwrap().clone();
            // Push ids on vm stack
            self.stack.push(pair.1);
            self.stack.push(pair.0);
            // Fetch rule for currently active pair
            let program = self.rules.get(
                &(self.interaction_net.atype(pair.0), self.interaction_net.atype(pair.1))
            );
            if program.is_some() {
                // Load rewriting instructions for rule
                self.code = program.unwrap().clone();
                self.pc = 0;
                // Execute rule
                self.run();
            }
        }
    }
}