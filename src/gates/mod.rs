mod factory;
mod primitives;
mod graph;
mod logics;

pub use factory::{GateFactory, GateFactoryFunction, Gate, PinKind, PinKey, PinValues};
#[macro_use] use crate::build_gate_function;
#[macro_use] use crate::connect;

use primitives::gate_nand;
use logics::{gate_or, gate_not};

impl GateFactory {
    pub fn new() -> GateFactory {
        let mut out = GateFactory::default();
        out.register(gate_not);
        out.register(gate_nand);
        out.register(gate_or);

        out
    }
}
