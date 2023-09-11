#[path = "../src/interaction/mod.rs"]
mod interaction;

#[path = "../src/interaction/vm.rs"]
mod vm;

mod unit_tests {
    use crate::interaction::{Agent, InteractionNet};
    use crate::vm;
    use crate::vm::{Instruction, VM};
    use crate::vm::Instruction::CONST;

    #[test]
    pub fn test_innet() {

        let mut innet = InteractionNet::new();
        innet.new_agent(1, 0);
        innet.new_agent(2, 0);
        innet.new_agent(3, 0);

        innet.bind_agents((false, false), 1, 2);
        innet.bind_agents((true, true), 1, 3);

        innet.bind_agents((false, false), 2, 3);

        innet.drop_agent(1);

        assert_eq!(innet.query_agent(2).ports.len(), 2);
        assert_eq!(innet.query_agent(3).ports.len(), 2);
    }

    #[test]
    pub fn test_net() {
        let mut innet = InteractionNet::new();
        innet.new_agent(1, 1);
        innet.new_agent(2, 1);

        innet.bind_agents((true, false), 1, 2);

        assert_eq!(innet.query_agent(1).id, 1);
        assert_eq!(innet.query_agent(1).atype, 1);

        assert_eq!(innet.query_agent(1), Agent { id: 1, atype: 1, ports: vec![2] });
        assert_eq!(innet.query_agent(2), Agent { id: 2, atype: 1, ports: vec![0, 1] });

        innet.unbind_agents(1, 2);

        assert_eq!(innet.query_agent(1), Agent { id: 1, atype: 1, ports: vec![0] });
        assert_eq!(innet.query_agent(2), Agent { id: 2, atype: 1, ports: vec![0] });
    }

    #[test]
    #[should_panic]
    pub fn test_panic() {
        let mut innet = InteractionNet::new();
        innet.new_agent(1, 1);
        innet.drop_agent(1);
        // Should panic here
        innet.query_agent(1);
    }

    #[test]
    pub fn test_vm_code_step() {
        let mut vm = VM::new();
        vm.code = vec![Instruction::NOP];
        vm.run();
        assert_eq!(vm.pc, 1);
    }

    #[test]
    #[should_panic]
    pub fn test_vm_run() {
        let mut vm = VM::new();
        vm.stack.push(1);
        vm.stack.push(1);
        vm.code = vec![vm::Instruction::NEW_AGENT];
        vm.run();
        assert_eq!(vm.interaction_net.query_agent(1), Agent{id: 1, atype: 1, ports: vec![0]});
        vm.code.push(CONST(1));
        vm.code.push(Instruction::DROP_AGENT);
        vm.run();
        // Panic here
        vm.interaction_net.query_agent(1);
    }

    #[test]
    pub fn test_vm_reduce() {
        let mut vm = VM::new();
        vm.new_rewrite((1, 1), vec![Instruction::POP(1), Instruction::POP(2)]);
        vm.interaction_net.new_agent(2, 1);
        vm.interaction_net.new_agent(3, 1);
        vm.interaction_net.bind_agents((true, true), 2, 3);
        vm.reduce();
        assert_eq!(vm.scratchpad[1], 2);
        assert_eq!(vm.scratchpad[2], 3);
    }

    #[test]
    pub fn test_vm_instructions() {
        let mut vm = VM::new();
        vm.code = vec![
            Instruction::GEN,
            Instruction::DUP,
            Instruction::CONST(1),
            Instruction::NEW_AGENT,
            Instruction::GEN,
            Instruction::DUP,
            Instruction::CONST(1),
            Instruction::NEW_AGENT,
            Instruction::POP(0),
            Instruction::POP(1),
            Instruction::PUSH(0),
            Instruction::PUSH(1),
            Instruction::CONST(0),
            Instruction::CONST(0),
            Instruction::BIND,
            Instruction::PUSH(0),
            Instruction::PUSH(1),
            Instruction::UNBIND,
        ];
        vm.run();

        assert_eq!(vm.interaction_net.heap.len(), 2);
        assert_eq!(vm.interaction_net.heap.get(&2).unwrap().ports.len(), 1);
        assert_eq!(vm.interaction_net.heap.get(&3).unwrap().ports.len(), 1);
        assert_eq!(vm.scratchpad[0], 3);
        assert_eq!(vm.scratchpad[1], 2);
    }
}