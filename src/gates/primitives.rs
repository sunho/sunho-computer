pub use super::factory::{GateFactory, GateFactoryFunction, Gate, PrimitiveGateImplementor, PinValues, PinKind, PinKey};
#[macro_use] use crate::build_gate_function;
#[macro_use] use crate::connect;

#[derive(Debug)]
struct NandImplementor { }

impl PrimitiveGateImplementor for NandImplementor {
    fn run(&mut self, inputs: PinValues) -> PinValues {
        let a = inputs.get("a", 0);
        let b = inputs.get("b", 0);
        let mut out =  PinValues::new();
        out.set("out", 0, !(a && b));
        return out;
    }
}

pub fn gate_nand() -> GateFactoryFunction {
    build_gate_function! {
        nand(a, b => out):
            | g: &mut Gate, f: &GateFactory | {
               g.primitive_implementor = Some(Box::new(NandImplementor {}))
            }
    }
}