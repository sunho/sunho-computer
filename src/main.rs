use std::{collections::{BTreeMap, HashMap}, rc::{Rc}, borrow::Cow};

struct Pin {
    name: String,
    index: i64,
    connected: Option<Rc<Pin>>,
    parent: String
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
struct PinKey {
    name: String,
    index: i64
}

impl PinKey {
    fn new(name: &str, index: i64) -> PinKey {
        PinKey { name: name.to_string(), index: index } 
    }
}


struct PinMap {
    internal: BTreeMap<PinKey, Rc<Pin>>,
    names: Vec<String>
}

impl PinMap {
    fn new() -> PinMap {
        PinMap {
            internal: BTreeMap::new(),
            names: Vec::new()
        }    
    }

    fn insert(&mut self, id: &str, name: &str, index: i64) {
        let pin = Pin {
            name: name.to_string(),
            index: index,
            connected: None,
            parent: id.to_string()
        };
        let name2 = name.to_string();
        self.names.push(name2);
        self.internal.insert(PinKey::new(&self.names[self.names.len()-1], index), Rc::new(pin));
        
    }

    fn find(&self, name: &str, index: i64) -> Option<Rc<Pin>> {
        match self.internal.get(&PinKey::new(name, index)) {
            None => None,
            Some(x) => Some(x.clone())
        }
    }
}

struct Gate {
    id: String,
    inputs: PinMap,
    outputs: PinMap,
    internal_pins: PinMap,
    gates: Vec<Gate>
}

impl Gate {
    fn new() -> Gate {
        Gate {
            id: "hello".to_string(),
            inputs: PinMap::new(),
            outputs: PinMap::new(),
            internal_pins: PinMap::new(),
            gates: Vec::new()
        }
    }

    fn insert_input_pin(&mut self, name: &str, index: i64) {
        self.inputs.insert(&self.id, name, index);
    }

    fn insert_output_pin(&mut self, name: &str, index: i64) {
        self.outputs.insert(&self.id, name, index);
    }

    fn insert_internal_pin(&mut self, name: &str, index: i64) {
        self.internal_pins.insert(&self.id, name, index);
    }

    fn find_input_pin(&self, name: &str, index: i64) -> Option<Rc<Pin>> {
        self.inputs.find(name, index)
    }
    
    fn find_output_pin(&self, name: &str, index: i64) -> Option<Rc<Pin>> {
        self.outputs.find(name, index)
    }

    fn find_internal_pin(&self, name: &str, index: i64) -> Option<Rc<Pin>> {
        self.internal_pins.find(name, index)
    }
}

macro_rules! build_gate {
    ($name:ident ($($input:ident $([ $input_size:tt ])?), * => $($output:ident $([ $output_size:tt ])?), *) { $($body:tt)* }) => (
        fn hello() -> Gate {

            let mut gate = Gate::new();

            $(
                if (stringify!($($input_size)?).is_empty()) {
                    gate.insert_input_pin(stringify!($input), 0);
                } else {
                    for i in 0..$($input_size)? {
                        gate.insert_input_pin(stringify!($input), i);
                    }
                }
                
            )*

            $(
                if (stringify!($($output_size)?).is_empty()) {
                    gate.insert_output_pin(stringify!($output), 0);
                } else {
                    for i in 0..$($output_size)? {
                        gate.insert_output_pin(stringify!($output), i);
                    }
                }    
            )*

            $($body)*

            return gate;
        }
    );
}

macro_rules! connect {
    ($w:expr, $g:expr, 
        $($name:ident = $pin:ident $([ $inner:tt $(.. $inner2:tt)?])?), * => 
        $($name2:ident = $pin2:ident $([ $inner21:tt $(.. $inner22:tt)?])?), *) => {{
        $(
            {
                let name = stringify!($name);
                let start = stringify!($($inner)?);
                let end = stringify!($($($inner2)?)?);
                if (start.is_empty()) {
                    
                }
            }
        )*
        $(
            {
                let start = stringify!($($inner21)?);
                let end = stringify!($($($inner22)?)?);
                if (start.is_empty()) {
                    println!("input: {} start", stringify!($name2));
                }
                
            }
        )*
    }};
}

build_gate! {
    hello_gate(a[3] => b) {
        connect!(gate, nand, a=a, b=b => out=out);
        connect!(gate, nand, a=a, b=b => out=out);
    }
}

fn main() {
    println!("Hello, world!");
    hello();

}   
