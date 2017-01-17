pub struct Add;
use mesh::mesh::Signal;
use mesh::mesh::Processor;

impl Processor for Add {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {
        let a: f32;
        let b: f32;
        match input[0] {
            Signal::Sound(x) => a = x,
        }

        match input[1] {
            Signal::Sound(x) => b = x,
        }
        vec![Signal::Sound(a + b)]
    }

    fn input_types_and_defaults(& self) -> Vec<Signal> {
        vec![Signal::Sound(0.0), Signal::Sound(0.0)]
    }

    fn output_types(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }
}

impl Add {
    pub fn new() -> Add {
        Add{}
    }
}
