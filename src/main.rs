use gates::{PinValues, GateFactory};

mod gates;


fn main() {
    let factory = GateFactory::new();
    let mut i = factory.build("or");
    let mut inputs = PinValues::new();
    inputs.set_binary("a", "0");
    inputs.set_binary("b", "0");
    let o = i.run(inputs);
    println!("{}", o);
}   
