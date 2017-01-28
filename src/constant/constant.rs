use mesh::mesh::Processor;
use mesh::mesh::Mesh;
use mesh::mesh::Signal;

pub struct Constant;


impl Processor for Constant {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {
        vec![input[0].clone()]
    }

    fn input_types_and_defaults(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }

    fn output_types(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }
    
    fn type_name(&self) -> String {
        String::from("Constant")        
    }
}

impl Constant {

    pub fn new() -> Constant {
        Constant {}
    }
}
