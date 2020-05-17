
use super::GateFactoryFunction;
#[macro_use] use crate::build_gate_function;
#[macro_use] use crate::connect;

pub fn gate_not() -> GateFactoryFunction {
    build_gate_function! {
        not(input => out):
            | g: &mut Gate, f: &GateFactory | {
                connect!(g, f, nand { a=input, b=input } => { out=out });
            }
    }
}

pub fn gate_or() -> GateFactoryFunction {
    build_gate_function! {
        or(a, b => out):
            | g: &mut Gate, f: &GateFactory | {
                connect!(g, f, not { input=a } => { out=nota });
                connect!(g, f, not { input=b } => { out=notb });
                connect!(g, f, nand { a=nota, b=notb } => { out=out });
            }
    }
}
