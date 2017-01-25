use mesh::mesh::Signal;
use mesh::mesh::Processor;

pub struct Intpipe {
    signal:            i64,
}

impl Processor for Intpipe {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {
        //first entry of the input array is Signal::Sound(end_amplitute).
        //second entry of the input array is Signal::Time(time).
        let sound: i64;
        match input[0]{
            Signal::Int(a) => sound = a,
            Signal::Sound(_)   => panic!(),
        }
        vec![Signal::Int(sound)]
    }

    fn input_types_and_defaults(&self) -> Vec<Signal> {
        vec![Signal::Int(0)]
    }

    fn output_types(&self) -> Vec<Signal> {
        vec![Signal::Int(0)]
    }

    fn type_name(&self) -> String {
        String::from("Dac")
    }
}

impl Intpipe {
    pub fn new() -> Intpipe {
        Intpipe {signal: 0}
    }
}
