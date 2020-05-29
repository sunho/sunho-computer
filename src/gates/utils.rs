use crate::gates::gate::PinKind;
use std::collections::BTreeSet;
use crate::gates::gate::{Pin};
use std::collections::BTreeMap;
use std::fmt;

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
pub struct PinMap {
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