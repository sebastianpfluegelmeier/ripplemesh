use mesh::mesh::Signal;
use std::vec::Vec;
use mesh::mesh::Processor;

pub struct Slope {
    signal:            f32,
    time:              f32,
    old_time:          f32,
    old_end_amplitude: f32
}

impl Processor for Slope {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {
        //first entry of the input array is Signal::Sound(end_amplitute).
        //second entry of the input array is Signal::Time(time).
        let sound: f32;
        match input[0]{

            Signal::Sound(a) => sound = a,

        }
        let output = sound; // just for now
        vec![Signal::Sound(output)]
    }

    fn input_types_and_defaults(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0), 
             Signal::Sound(0.0),
             Signal::Sound(0.0),
             Signal::Sound(0.0)]
    }

    fn output_types(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }

}

impl Slope {
    pub fn new() -> Slope {
        Slope{signal: 0.0, time: 0.0, old_time: 0.0, old_end_amplitude: 0.0}
    }
}
