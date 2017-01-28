#![allow(dead_code)]

extern crate portaudio;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::option;
use std::vec::Vec;
use std::collections::LinkedList;
use std::collections::HashMap;
use pipe::pipe::Pipe;
use add::add::Add;
use mult::mult::Mult;
use intpipe::intpipe::Intpipe;
use dac::dac::Dac;
use sine::sine::Sine;
use mesh::mesh::Processor;
use mesh::mesh::Mesh;
use mesh::mesh::Signal;
use dummy::dummy::Dummy;

use self::portaudio as pa;

pub const SAMPLERATE: f64 = 44100.0;
pub const CHANNELS: i32 = 1;
pub const FRAMES_PER_BUFFER: u32 = 64;

pub enum CallbackMessage {
    Processor(Box<Processor>),
    Connections((Vec<Vec<Vec<(usize, usize)>>>, Vec<usize>, Vec<usize>)),
    Constant(usize, f64),
}

pub struct Engine {
    pub processors: Vec<Box<Processor>>,
    pub input_buffers: Vec<Vec<Signal>>, // computed signals are stored here until
                                     //they get processed.
    //[out_processor][out_plug][connection](in_processor, in_plug)
    pub adjacency_list: Vec<Vec<Vec<(usize, usize)>>>,
    pub topologically_ordered: Vec<usize>,
    pub io: Vec<usize>,
    pub rec: Receiver<CallbackMessage>,
}

impl Engine {

    fn add_processor(&mut self, processor: Box<Processor>) {
        self.processors.push(processor);
    }

    fn remove_processor(&mut self, index: usize) {
        self.processors[index] = Box::new(Dummy::new());
    }

    fn update_connections(&mut self, adjacency_list: Vec<Vec<Vec<(usize, usize)>>>, topologically_ordered: Vec<usize>, io: Vec<usize>) {
        self.adjacency_list = adjacency_list;
        self.topologically_ordered = topologically_ordered;
        self.io = io;
    }

    pub fn new(receiver: Receiver<CallbackMessage>) -> Engine {
        Engine {
            processors: Vec::new(),
            input_buffers: Vec::new(),
            adjacency_list: Vec::new(),
            topologically_ordered: Vec::new(),
            io: Vec::new(),
            rec: receiver,
        }
    }

    pub fn set_constant(&mut self, constant: usize, value: f64) {
        if self.processors[constant].type_name() == "Constant" {
            self.input_buffers[constant][0] = Signal::Sound(value);
        } else {
            println!("this processor is no constant");
        }
    }

    pub fn process(&mut self) -> Vec<f32> {
        match self.rec.try_recv().unwrap() {
            CallbackMessage::Processor(a) => self.add_processor(a),
            CallbackMessage::Connections((adj, topo, io)) => self.update_connections(adj, topo, io),
            CallbackMessage::Constant(a, b) => self.set_constant(a, b),
        }
        {
            let mut processors = &mut self.processors;
            for processor_num in &self.topologically_ordered {
                let processor_num_two = *processor_num;
                let processor_num_three = *processor_num;
                let mut boxed_processor: &mut Box<Processor> = &mut processors[*processor_num];
                let mut processor = boxed_processor;
                let result: Vec<Signal>;
                {
                    let input : &Vec<Signal> = &self.input_buffers[processor_num_two];
                    result = processor.process(input);
                }
                let connections: &Vec<Vec<(usize, usize)>> = 
                    &self.adjacency_list[processor_num_three];
                for (plug_num, plug) in connections.iter().enumerate() {
                    for other in plug {
                        let (other_processor_num, other_input) = *other;
                        self.input_buffers[other_processor_num][other_input] = result[plug_num].clone();
                    }
                }
            }
        }
        for io_processor_num in &self.io {
            let io_processor = &self.processors[*io_processor_num];
            if io_processor.type_name() == "Dac" {
                //TODO: implement properly, only supports one channel
                let signal: f32;
                match self.input_buffers[*io_processor_num][0] {
                    Signal::Sound(a) => signal = a as f32,
                    _                => panic!(),
                }
                return vec![signal];
            }
        }
        return vec![0.0];
    }

}

