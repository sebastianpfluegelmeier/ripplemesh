use mesh::mesh::Signal;
use mesh::mesh::Processor;
use mesh::mesh::SAMPLERATE;
use std::f64::consts::PI;

pub struct Sine {
    phase: f64,
}

impl Sine {
    pub fn new() -> Sine {
        Sine {phase: 0.0}
    }
}

impl Processor for Sine {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {
        let freq: f64;
        let output: f64;
        match input[0] {
            Signal::Sound(a) => freq = a,
            _                => panic!(),
        }
        output = f64::sin(self.phase);
        self.phase += (freq/SAMPLERATE) * PI; 
        
        vec![Signal::Sound(output)]
    }

    fn input_types_and_defaults(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }

    fn output_types(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }
}
