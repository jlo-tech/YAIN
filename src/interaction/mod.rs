pub mod vm;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    pub id: u64,        /* NOTE: 0 is not a valid id!!! */
    pub atype: u64,
    pub ports: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct InteractionNet {
    pub cnt: u64,
    pub heap: HashMap<u64, Agent>,
    pub active_pairs: Vec<(u64, u64)>,
}

impl InteractionNet {

    pub fn new() -> Self {
        InteractionNet {
            cnt: 1,
            heap: HashMap::new(),
            active_pairs: Vec::new(),
        }
    }

    pub fn gen_id(&mut self) -> u64 {
        self.cnt = self.cnt + 1;
        return self.cnt;
    }

    // Create agent
    pub fn new_agent(&mut self, id: u64, atype: u64) {
        self.heap.insert(id, Agent{id: id, atype: atype, ports: vec![0]});
    }

    // Remove agent
    pub fn drop_agent(&mut self, id: u64) {
        // Unbind agent from all others
        let local_agent = self.query_agent(id);
        for port in &local_agent.ports {
            self.unbind_agents(id, port.clone());
        }
        // Remove agent from heap
        self.heap.remove(&id);
    }

    // Get Agent
    pub fn query_agent(&self, aid: u64) -> Agent {
        return self.heap.get(&aid).unwrap().clone();
    }

    // Get type of agent
    pub fn atype(&self, aid: u64) -> u64 {
        return self.query_agent(aid).atype;
    }

    // Get arity of agent
    pub fn arity(&self, aid: u64) -> u64 {
        return self.query_agent(aid).ports.len() as u64;
    }

    // Connect two agents
    pub fn bind_agents(&mut self, principals: (bool, bool), aid0: u64, aid1: u64) {
        // Check for invalid id
        if (aid0 == 0) || (aid1 == 0) {
            return;
        }
        // Obtain local copies
        let mut lcopy = self.heap.get(&aid0).unwrap().clone();
        let mut rcopy = self.heap.get(&aid1).unwrap().clone();
        // Add connection from aid0 to aid1
        if principals.0 {
            lcopy.ports[0] = aid1
        } else {
            lcopy.ports.push(aid1);
        }
        // Add connection from aid1 to aid0
        if principals.1 {
            rcopy.ports[0] = aid0
        } else {
            rcopy.ports.push(aid0);
        }
        // Push active pair if new one gets created
        if principals.0 && principals.1 {
            self.active_pairs.push((aid0, aid1));
        }
        // Update Agents
        self.heap.insert(lcopy.id, lcopy);
        self.heap.insert(rcopy.id, rcopy);
    }

    // Remove connection of two agents
    pub fn unbind_agents(&mut self, aid0: u64, aid1: u64) {
        // Check for invalid id
        if (aid0 == 0) || (aid1 == 0) {
            return;
        }
        // Obtain local copies
        let mut lcopy = self.heap.get(&aid0).unwrap().clone();
        let mut rcopy = self.heap.get(&aid1).unwrap().clone();
        // Check if it is a principal connection
        if lcopy.ports.get(0).unwrap().clone() == aid1 {
            // In case overwrite principal port
            lcopy.ports[0] = 0;
        } else {
            // If not delete auxiliary port
            lcopy.ports.retain(|e| e.clone() != aid1);
        }
        // Check if it is a principal connection
        if rcopy.ports.get(0).unwrap().clone() == aid0 {
            // In case overwrite principal port
            rcopy.ports[0] = 0;
        } else {
            // If not delete auxiliary port
            rcopy.ports.retain(|e| e.clone() != aid0);
        }
        // Update Agents
        self.heap.insert(lcopy.id, lcopy);
        self.heap.insert(rcopy.id, rcopy);
    }
}