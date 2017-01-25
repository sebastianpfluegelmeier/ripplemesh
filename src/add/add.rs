pub struct Add;
use mesh::mesh::Signal;
use mesh::mesh::Processor;

impl Processor for Add {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {
        let a: f64;
        let b: f64;
        match input[0] {
            Signal::Sound(x) => a = x,
            Signal::Int(_)   => panic!(),
        }

        match input[1] {
            Signal::Sound(x) => b = x,
            Signal::Int(_)   => panic!(),
        }
        vec![Signal::Sound(a + b)]
    }

    fn input_types_and_defaults(& self) -> Vec<Signal> {
        vec![Signal::Sound(0.0), Signal::Sound(0.0)]
    }

    fn output_types(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }

    fn type_name(&self) -> String {
        String::from("Add")
    }
}

impl Add {
    pub fn new() -> Add {
        Add{}
    }
}
