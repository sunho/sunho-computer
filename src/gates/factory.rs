use crate::gates::gate::Gate;
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
        
        use crate::gates::{GateFactory, GateFactoryFunction, Gate, PinKind, PinKey, Pin};
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