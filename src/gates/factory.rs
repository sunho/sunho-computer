use std::{collections::{BTreeMap, HashMap, BTreeSet}, rc::{Rc}, borrow::Cow, fmt};
use super::graph::Graph;

pub struct GateFactory {
    factory_funcs: BTreeMap<String, fn(&GateFactory) -> Gate>
}

impl GateFactory {
    pub fn default() -> GateFactory {
        GateFactory {
            factory_funcs: BTreeMap::new()
        }
    }

    pub fn register(&mut self, func: fn() -> GateFactoryFunction) {
        let tmp = func();
        self.factory_funcs.insert(tmp.name.to_string(), tmp.generator);
    }

    pub fn build(&self, name: &str) -> Gate {
        self.factory_funcs.get(name).unwrap()(self)
    }
}

pub struct GateFactoryFunction {
    pub name: String,
    pub generator: fn(&GateFactory) -> Gate
}

#[derive(Debug)]
enum Connection {
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

// Externally, Gate receives inputs and return outputs (not internals) 
// 
// Gate input pins are connected to its child gate's inputs pins (ToGate)
// 
// ToGate specifies the input set
// 
// Input set:
//   variables from inputs
//   variables from internal pins
//   variables from outpus pins
// 
// Child gates outputs are connected to parent internal pins or outputs pin
// 
// returns only outputs (it's not child buisiness)
// 
// run:
//   receives input 
//   generate input sets (considers input pins of children)
//   forward input sets to gate and run
//   store internal state (considers output pins of children)
//   store output state (considers output pins of children)


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
    connection: Connection
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Clone)]
pub struct PinKey {
    name: String,
    index: i64
}

impl PinKey {
    pub fn new(name: &str, index: i64) -> PinKey {
        PinKey { name: name.to_string(), index: index } 
    }
}

#[derive(Debug)]
struct PinMap {
    internal: BTreeMap<PinKey, Pin>,
    names: Vec<String>
}

#[derive(Debug, Clone)]
pub struct PinValues {
    map: BTreeMap<PinKey, bool>,
    names: BTreeSet<String>
}

impl PinValues {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            names: BTreeSet::new()
        }
    }

    pub fn set_binary(&mut self, name: &str, value: &str)  {
        for (i, c) in value.chars().enumerate() {
            if c == '0' {
                self.set(name, i as i64, false);
            } else {
                self.set(name, i as i64, true);
            }
        };
    }

    pub fn get(&self, name: &str, index: i64) -> bool {
        *self.map.get(&PinKey::new(name, index)).unwrap()
    }

    pub fn set(&mut self, name: &str, index: i64, value: bool) {
        self.map.insert(PinKey::new(name, index), value);
        self.names.insert(name.to_string());
    }
}

impl fmt::Display for PinValues {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = Vec::new();
        for name in &self.names {
            let mut out = Vec::new();
            let mut i = 0;
            while true {
                let a = self.map.get(&PinKey::new(&name, i));
                if a.is_none() {
                    break;
                }
                if *a.unwrap() == true {
                    out.push("1");
                } else {
                    out.push("0");
                }
                i += 1;
            }
            res.push([name.clone(), out.join("")].join(":"));
        }
        write!(f, "{}", res.join("\n"))
    }
}

impl PinMap {
    fn new() -> PinMap {
        PinMap {
            internal: BTreeMap::new(),
            names: Vec::new()
        }    
    }

    fn insert(&mut self, id: &str, kind: PinKind, name: &str, size: i64, index: i64) {
        let pin = Pin {
            name: name.to_string(),
            index: index,
            kind: kind,
            size: size,
            connection: Connection::None
        };
        let name2 = name.to_string();
        self.names.push(name2);
        self.internal.insert(PinKey::new(&self.names[self.names.len()-1], index), pin);
        
    }

    fn get(&self, name: &str, index: i64) -> Option<&Pin> {
        match self.internal.get(&PinKey::new(name, index)) {
            None => None,
            Some(x) => Some(x)
        }
    }

    fn get_mut(&mut self, name: &str, index: i64) -> Option<&mut Pin> {
        match self.internal.get_mut(&PinKey::new(name, index)) {
            None => None,
            Some(x) => Some(x)
        }
    }
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
    pins: PinMap,
    compiled_plans: Option<Vec<GateRunPlan>>,
    pub gates: Vec<Gate>,
    pub primitive_implementor: Option<Box<dyn PrimitiveGateImplementor>>
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
            compiled_plans: None,
            gates: Vec::new(),
            primitive_implementor: None
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
        self.compiled_plans = Some(out);
        Ok(())
    }

    pub fn run(&mut self, inputs: PinValues) -> PinValues {
        match self.primitive_implementor.as_mut() {
            Some(x) => x.run(inputs),
            None => {
                let mut temp_values = inputs.clone();
                let mut output_values = PinValues::new();

                match &self.compiled_plans {
                    Some(runs) => {
                        for run in runs {
                            let gate = self.gates.get_mut(run.gate_index as usize).unwrap();
                            let mut xx = PinValues::new();
                            run.reads.iter()
                                .for_each(| (parent, child) | { 
                                    let x = temp_values.get(&parent.name, parent.index);
                                    xx.set(&child.name, child.index, x);
                                });

                            let res = gate.run(xx);
                            run.write_internals.iter()
                                .for_each(| (parent, child) | {
                                    let x = res.get(&child.name, child.index);
                                    temp_values.set(&parent.name, parent.index, x);
                                });
                            
                            run.write_outputs.iter()
                                .for_each(| (parent, child) | {
                                    let x = res.get(&child.name, child.index);
                                    output_values.set(&parent.name, parent.index, x);
                                });
                        }
                    },
                    None => {
                        return output_values;
                    }
                }
                output_values
            }
        }
    }
}

#[macro_export]
macro_rules! build_gate_function {
    ($name:ident ($($input:ident $([ $input_size:tt ])?), * => $($output:ident $([ $output_size:tt ])?), *): $next:expr) => (
        (|| {
            use crate::gates::factory::{GateFactory, GateFactoryFunction, Gate, PinKind, PinKey, Pin};
            GateFactoryFunction {
                name: stringify!($name).to_string(),
                generator: |f: &GateFactory| -> Gate {
                    let mut gate = Gate::new(stringify!($name));
                    $(
                        if (stringify!($($input_size)?).is_empty()) {
                            gate.insert_pin(PinKind::Input, stringify!($input), 1, 0);
                        } else {
                            let mut x = 0;
                            for i in 0..$($input_size)? {
                                x += 1;
                            }
                            for i in 0..$($input_size)? {
                                gate.insert_pin(PinKind::Input, stringify!($input), x, i);
                            }
                        }
                        
                    )*

                    $(
                        if (stringify!($($output_size)?).is_empty()) {
                            gate.insert_pin(PinKind::Output, stringify!($output), 1, 0);
                        } else {
                            let mut x = 0;
                            for i in 0..$($output_size)? {
                                x += 1;
                            }
                            for i in 0..$($output_size)? {
                                gate.insert_pin(PinKind::Output, stringify!($output), x, i);
                            }
                        }    
                    )*
                    $next(&mut gate, f);
                    if let Err(x) = gate.compile() {
                        panic!(x)
                    }
                    return gate;
                }
            }
        })()
    )
}

#[macro_export]
macro_rules! connect {
    () => ((0,0));
    ($($name:ident $([($name_index:tt)*])? = $pin:ident $([$($pin_index:tt)*])?),*) => (
        (|| {
            let mut out = Vec::new();
            $( 
                let range: (usize, usize) =  connect!($($($name_index)*)?);
                let range2: (usize, usize) =  connect!($($($pin_index)*)?);
                out.push(((stringify!($name), range), (stringify!($pin), range2)));
            )*
            out
        })()
    );
    ($index_start:expr, $index_end:expr) => {{
        (|| {
            ($index_start, $index_end)
        })()
    }};
    ($index_start:expr) => {{
        (|| {
            ($index_start, $index_start + 1)
        })()
    }};
    ($g:ident, $f:ident, $gate_name:ident { $($input:tt)* } => { $($output:tt)* })=> {{
        
        use crate::gates::factory::{GateFactory, GateFactoryFunction, Gate, PinKind, PinKey, Pin};
        let gate = $f.build(stringify!($gate_name));
        let gate2 = $f.build(stringify!($gate_name));
        let gi = $g.add_gate(gate2);
        let inputs = connect!($($input)*);
        let outputs = connect!($($output)*);
        let mut connect = | child: &(&str, (usize, usize)), parent: &(&str, (usize, usize))| {

            let ( pname, mut prange ) = parent;
            let ( cname, mut crange ) = child;
    
            
            if crange.0 == crange.1 {
                let tmp = cname.to_string();
                let size = gate.get_pin(&cname.to_string(), 0).unwrap().size;
                crange = (0,size as usize);
            }

            if !$g.exists_pin(pname, 0) {
                for i in 0..(crange.1 - crange.0) {
                    $g.insert_pin(PinKind::Internal, pname, (crange.1 - crange.0) as i64, i as i64);
                }   
            }

            if prange.0 == prange.1 {
                let size = $g.get_pin(pname, 0).unwrap().size;
                prange = (0,size as usize);
            }

            for i in 0..(crange.1-crange.0){
                $g.connect_pins(gi,  &PinKey::new(pname, (prange.0 + i) as i64), &PinKey::new(cname, (crange.0 + i) as i64));
            }
        };

        for i in &inputs {
            let (x, y) = i;
            connect(x,y);
        }

        for i in &outputs {
            let (x, y) = i;
            connect(x,y);
        }
    }};
}