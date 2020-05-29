
// tick: write out[t]
// tock: run combintional circuits & update out[t]
// 
// EX
// out[t+1] -> Bit() -> out[t] -> Comb
// tick: output all clocked pins (out of DFF)
// tock: run Comb & run Bit
// 
// run -> DFF -> clocked_run
//
// DFF -> clocked 
// 
// Invariants
// Comb_t = f(Bit_t) tick 
// Bit_t+1 = g(Comb_t(.)) tock
// 
// Inside:
// Tick: run every gate that is not connected with Bit_t+1
// Tock: run every gate that is connected with Bit_t+1
// 
// Outside:
// Tick: run inside tick according to the sequence (but only consider the gates that ticks)
// New dependency: gates don't need in and load in tick stage
// 
// Tock: run inside tock regardless of order (t1 only depends on internal state)
//
// Bit_t+1 connected with 

use std::collections::BTreeMap;
use crate::gates::utils::PinValues;
use crate::gates::utils::PinMap;
use crate::gates::utils::PinKey;

#[derive(Debug)]
pub enum Connection {
    None,
    ToParent(String, i64),
    ToChild(String, i64)
}

#[derive(Debug)]
pub enum PinKind {
    Input,
    Internal,
    Output
}

pub enum GateValidationError {
    InvalidPinConnection,
    PinNotExists
}


#[derive(Debug)]
pub struct Pin {
    pub name: String,
    pub index: i64,
    pub size: i64,
    pub kind: PinKind,
    pub clocked: bool,
    connection: Connection
}

#[derive(Debug, Clone)]
struct GateRunPlan {
    gate_index: i64,
    // parent -> child
    reads: Vec<(PinKey, PinKey)>,
    write_internals: Vec<(PinKey, PinKey)>,
    write_outputs: Vec<(PinKey, PinKey)>
}

#[derive(Debug)]
pub struct Gate {
    pub name: String,
    pub id: String,
    pub gates: Vec<Gate>,
    pub primitive_implementor: Option<Box<dyn PrimitiveGateImplementor>>,
    pins: PinMap,
    temp_values: PinValues,
    compiled_tick_plans: Option<Vec<GateRunPlan>>,
    compiled_tock_plans: Option<Vec<GateRunPlan>>
}

pub trait PrimitiveGateImplementor: std::fmt::Debug{
    fn run(&mut self, inputs: PinValues) -> PinValues;
}

impl Gate {
    pub fn new(name: &str) -> Gate {
        Gate {
            name: name.to_string(),
            id: "hello".to_string(),
            pins: PinMap::new(),
            compiled_tick_plans: None,
            compiled_tock_plans: None,
            gates: Vec::new(),
            primitive_implementor: None,
            temp_values: PinValues::new()
        }
    }

    pub fn add_gate(&mut self, gate: Gate) -> usize {
        self.gates.push(gate);
        self.gates.len() - 1
    }

    pub fn connect_pins(&mut self, gate_index: usize, pin: &PinKey, child_pin: &PinKey) -> Result<(), GateValidationError> {
        let x = match self.pins.get_mut(&pin.name, pin.index) {
            Some(x) => x,
            None => {
                return Err(GateValidationError::PinNotExists);
            }
        };

        let gate = match self.gates.get_mut(gate_index) {
            Some(x) => x,
            None => {
                return Err(GateValidationError::PinNotExists);
            }
        };
        let y = match gate.pins.get_mut(&child_pin.name, child_pin.index) {
            Some(x) => x,
            None => {
                return Err(GateValidationError::PinNotExists);
            }
        };

        x.connection = Connection::ToChild(y.name.clone(), y.index);
        y.connection = Connection::ToParent(x.name.clone(), x.index);
        Ok(())
    }

    pub fn get_pin(&self, name: &str, index: i64) -> Option<&Pin> {
        self.pins.get(name, index)
    }

    pub fn insert_pin(&mut self, kind: PinKind, name: &str, size: i64, index: i64) {
        self.pins.insert(&self.id, kind, name, size, index);
    }

    pub fn exists_pin(&self, name: &str, index: i64) -> bool {
        !self.pins.get(name, index).is_none()
    }

    pub fn compile(&mut self) -> Result<(), GateValidationError> {
        let mut graph = Graph::new();
        for i in  0..self.gates.len() {
            graph.add_node(i as i64);
        }
        let mut runs = Vec::new();
        let mut read_from_internal: BTreeMap<PinKey, Vec<i64>> = BTreeMap::new();
        let mut write_to_internal: BTreeMap<PinKey, i64> = BTreeMap::new();
        for i in  0..self.gates.len() {
            let gate = self.gates.get(i).unwrap();
            let mut reads = Vec::new();
            let mut write_internals = Vec::new();
            let mut write_outputs = Vec::new();
            for (_, pin) in &gate.pins.internal {
                if let Connection::ToParent(name, index) = &pin.connection {
                    let parent_key = PinKey::new(&name, *index);
                    let child_key = PinKey::new(&pin.name, pin.index);
                    let parent_pin = match self.pins.get(name, *index) {
                        Some(x) => x,
                        None => { return Err(GateValidationError::InvalidPinConnection) }
                    };
                    if let PinKind::Input = pin.kind {
                        match parent_pin.kind {
                            PinKind::Internal => {
                                read_from_internal.entry(parent_key.clone()).or_insert(Vec::new());
                                read_from_internal.get_mut(&parent_key).unwrap().push(i as i64);
                            },
                            PinKind::Input => { },
                            PinKind::Output => {
                                 return Err(GateValidationError::InvalidPinConnection);
                            }
                        };
                        reads.push((parent_key, child_key));
                    } else if let PinKind::Output = pin.kind {
                       match parent_pin.kind {
                           PinKind::Internal => {
                                write_internals.push((parent_key.clone(), child_key.clone()));
                                write_to_internal.insert(parent_key, i as i64);
                           },
                           PinKind::Output => {
                                write_outputs.push((parent_key, child_key));
                           },
                           PinKind::Input => {
                                return Err(GateValidationError::InvalidPinConnection);
                           }
                       };
                    }
                }
            }
            runs.push(GateRunPlan {
                gate_index: i as i64,
                reads: reads,
                write_internals: write_internals,
                write_outputs: write_outputs
            });
        }


        for (key, reads) in &read_from_internal {
            let node = match write_to_internal.get(key) {
                Some(x) => *x,
                None => { return Err(GateValidationError::InvalidPinConnection) }
            };
            for r in reads {
                graph.add_edge(*r, node);
            }
        }
        let results = match graph.topological_sort_with_cycle_detection() {
            Ok(x) => x,
            Err(_) => { return Err(GateValidationError::InvalidPinConnection) }
        };

        let mut out: Vec<GateRunPlan> = Vec::new();
        for i in results {
            out.push(runs[i as usize].clone());
        }
        self.compiled_tock_plans = Some(out);
        Ok(())
    }

    pub fn tick(&mut self, inputs: PinValues) {
        self.temp_values = inputs;
        match &self.compiled_tick_plans {
            Some(runs) => {
                self.run(runs)
            },
            None => PinValues::new()
        };
    }

    pub fn tock(&mut self) -> PinValues {
        match &self.compiled_tock_plans {
            Some(runs) => {
                self.run(runs)
            },
            None => PinValues::new()
        }
    }

    fn run(&mut self, runs: &Vec<GateRunPlan>, ) -> PinValues {
        match self.primitive_implementor.as_mut() {
            Some(x) => x.run(self.temp_values),
            None => {
                let mut output_values = PinValues::new();
                for run in runs {
                    let gate = self.gates.get_mut(run.gate_index as usize).unwrap();
                    let mut xx = PinValues::new();
                    run.reads.iter()
                        .for_each(| (parent, child) | { 
                            let x = self.temp_values.get(&parent.name, parent.index);
                            xx.set(&child.name, child.index, x);
                        });

                    let res = gate.tock();
                    run.write_internals.iter()
                        .for_each(| (parent, child) | {
                            let x = res.get(&child.name, child.index);
                            self.temp_values.set(&parent.name, parent.index, x);
                        });
                    
                    run.write_outputs.iter()
                        .for_each(| (parent, child) | {
                            let x = res.get(&child.name, child.index);
                            output_values.set(&parent.name, parent.index, x);
                        });
                }
                output_values
            }
        }
    }
}