extern crate portaudio;

use mesh::mesh::Signal;
use mesh::mesh::Processor;
use std::f64::consts::PI;

pub struct Dac;

impl Processor for Dac {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {
        vec![]
    }
    fn input_types_and_defaults(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }
    fn output_types(&self) -> Vec<Signal> {
        vec![]
    }

    fn type_name(&self) -> String {
        String::from("Dac")
    }

}

impl Dac {
    pub fn new() -> Dac {
        Dac
    }
}

