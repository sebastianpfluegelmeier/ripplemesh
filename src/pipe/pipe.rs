use mesh::mesh::Signal;
use mesh::mesh::Processor;

pub struct Pipe {
    signal:            f32,
}

impl Processor for Pipe {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {
        //first entry of the input array is Signal::Sound(end_amplitute).
        //second entry of the input array is Signal::Time(time).
        let sound: f32;
        match input[0]{
            Signal::Sound(a) => sound = a,
        }
        vec![Signal::Sound(sound)]
    }

    fn input_types_and_defaults(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }

    fn output_types(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }
}

impl Pipe {
    pub fn new() -> Pipe {
        Pipe {signal: 0.0}
    }
}