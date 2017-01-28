use mesh::mesh::Processor;
use mesh::mesh::Mesh;
use mesh::mesh::Signal;

pub struct Dummy;

impl Processor for Dummy {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {
        vec![]
    }

    fn input_types_and_defaults(&self) -> Vec<Signal> {
        vec![]
    }

    fn output_types(&self) -> Vec<Signal> {
        vec![]
    }
    
    fn type_name(&self) -> String {
        String::from("Dummy")        
    }
}

impl Dummy {
    pub fn new() -> Dummy {
        Dummy{}
    }
}
