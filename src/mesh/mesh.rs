#![allow(dead_code)]

extern crate portaudio;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::option;
use std::vec::Vec;
use std::collections::LinkedList;
use std::collections::HashMap;
use slope::slope::Slope;
use pipe::pipe::Pipe;
use add::add::Add;
use mult::mult::Mult;
use intpipe::intpipe::Intpipe;
use dac::dac::Dac;
use sine::sine::Sine;

use self::portaudio as pa;

pub const SAMPLERATE: f64 = 44100.0;
pub const CHANNELS: i32 = 1;
pub const FRAMES_PER_BUFFER: u32 = 64;

#[test]
fn sine_test() {
    let mut sine = Sine::new();
    let mut signal: Signal = Signal::Sound(0.0);
    for i in 0 .. 44100 {
        signal = sine.process(&vec![Signal::Sound(1.0)])[0].clone();
    }
    match signal {
        Signal::Sound(a) => assert!((a < 0.0001) && (a > -(0.0001))),
        _                => panic!(),
    }
    for i in 0 .. 22050 {
        signal = sine.process(&vec![Signal::Sound(1.0)])[0].clone();
    }
    match signal {
        Signal::Sound(a) => assert!((a > -1.0001) && (a < -0.9999)),
        _                => panic!(),
    }
}

#[test]
fn dac_test() {
    let mut mesh: Mesh = Mesh::new();
    mesh.add_processor(Sine::new());
    mesh.add_processor(Sine::new());
    mesh.input_buffers[1][0] = 0.5;
    mesh.add_processor(Mult::new());
    mesh.add_processor(Dac::new());
    mesh.connect((0, 0), (2, 0));
    mesh.connect((1, 0), (2, 1));
    mesh.connect((2, 0), (3, 0));
    mesh.run();
}

#[test]
fn test_types() {
    let mut mesh: Mesh = Mesh::new();
    for _ in 0..3 {
        mesh.add_processor(Pipe::new());
    }
    for _ in 0..3 {
        mesh.add_processor(Intpipe::new());
    }
    mesh.connect((0, 0), (1, 0));
    mesh.connect((1, 0), (2, 0));
    mesh.connect((3, 0), (4, 0));
    let mut connections_match = mesh.connect((4, 0), (5, 0));
    assert!(connections_match);
    connections_match = mesh.connect((0, 0), (4, 0));
    assert!(!connections_match);
}

#[test]
fn test_adder() {
    let mut mesh: Mesh = Mesh::new();
    for i in 0..5 {
        mesh.add_processor(Add::new());
    }
    mesh.connect((0, 0), (1, 0));
    mesh.connect((1, 0), (2, 1));
    mesh.connect((2, 0), (3, 0));
    mesh.connect((0, 0), (3, 1));
    mesh.connect((3, 0), (4, 0));
    mesh.input_buffers[0][0] = Signal::Sound(1.0);
    mesh.input_buffers[1][0] = Signal::Sound(2.0);
    print_input_buffers(&mesh);
    println!("");
    mesh.process();
    print_input_buffers(&mesh);
    match mesh.input_buffers[4][0] {
        Signal::Sound(a) => assert!(a == 2.0),
        Signal::Int(_)   => panic!(),
    }

}

#[test]
fn test_process() {
    let mut mesh: Mesh = Mesh::new();
    for i in 0..3 {
        mesh.add_processor(Pipe::new());
    }
    mesh.connect((0, 0), (1, 0));
    mesh.connect((1, 0), (2, 0));

    mesh.input_buffers[0][0] = Signal::Sound(1.0);
    print_input_buffers(&mesh);
    mesh.process();
    print_input_buffers(&mesh);
    match mesh.input_buffers[1][0] {
        Signal::Sound(a) => assert!(a == 1.0),
        Signal::Int(_)   => panic!(),
    }
}

fn print_input_buffers(mesh: &Mesh) {
    for (i, device) in mesh.input_buffers.iter().enumerate() {
        for (j, connection) in device.iter().enumerate() {
            let value: f64;
            match *connection {
                Signal::Sound(a) => value = a,
                Signal::Int(_)   => panic!(),
            }
            println!("{}:{}: {}", i, j, value);
        }
    }

}

#[test]
fn test_order_topologically() {
    let mut mesh: Mesh = Mesh::new();
    for i in 0..10 {
        mesh.add_processor(Slope::new());
    }
    
    mesh.connect((0, 0), (1, 0));
    mesh.connect((1, 0), (2, 0));
    mesh.connect((2, 0), (3, 0));
    mesh.connect((3, 0), (4, 0));
    mesh.connect((2, 0), (4, 0));
    mesh.connect((3, 0), (5, 0));
    mesh.connect((1, 0), (4, 0));
    mesh.connect((0, 0), (3, 0));

    mesh.order_topologically();
    let mut topo: Vec<usize> = Vec::new();

    match mesh.topologically_ordered {
        Some(x) => topo = x,
        None => panic!(),
    }

    println!("topo:");
    for i in topo {
        println!("{}",i);
    }
}

pub trait Processor {
    fn process(self: &mut Self, input: &Vec<Signal>) -> Vec<Signal>;
    fn input_types_and_defaults(self: &Self) -> Vec<Signal>;
    fn output_types(self: &Self) -> Vec<Signal>;
    fn type_name(self: &Self) -> String;
}


pub enum Signal {
    Sound(f64),
    Int(i64),
}

impl Clone for Signal {
    fn clone(&self) -> Signal {
        match *self {
            Signal::Sound(a) => return Signal::Sound(a),
            Signal::Int(_)   => panic!(),
        }
    }
}

pub struct Mesh {
    processors: Vec<Box<Processor>>,
    input_buffers: Vec<Vec<Signal>>, // computed signals are stored here until
                                     //they get processed.
    //[out_processor][out_plug][connection](in_processor, in_plug)
    adjacency_list: Vec<Vec<Vec<(usize, usize)>>>,
    topologically_ordered: Option<Vec<usize>>,
    io: Vec<usize>,
    tx: Sender<f32>,
}

impl Mesh {

    fn new() -> Mesh {
        let (ttx, _) = mpsc::channel();
        Mesh {
            processors: Vec::new(),
            input_buffers: Vec::new(),
            adjacency_list: Vec::new(),
            topologically_ordered: Option::Some(Vec::new()),
            io: Vec::new(),
            tx: ttx,
        }
    }

    fn add_processor<T: 'static + Processor> 
      (self: &mut Mesh, processor: T) -> usize {

        self.adjacency_list.push(Vec::new());
        for _ in 0..processor.output_types().len() {
            let mut last: Vec<Vec<(usize, usize)>> = Vec::new();

            {
                match self.adjacency_list.pop() {
                    Some(x) => last = x,
                    None => println!("this should not happen"),
                }
                last.push(Vec::new());
            }
            self.adjacency_list.push(last);
        }
        let boxed_processor: Box<Processor> = Box::new(processor);
        self.input_buffers.push(boxed_processor.input_types_and_defaults());
        if boxed_processor.type_name() == "Dac" {
            self.io.push(self.adjacency_list.len() - 1);
        }
        self.processors.push(boxed_processor);
        self.order_topologically();
        self.adjacency_list.len() - 1
    }

    fn run(&mut self) -> Result<(), pa::Error> {
        let pa = try!(pa::PortAudio::new());

        let mut settings = 
            try!(pa.default_output_stream_settings(
                    CHANNELS, SAMPLERATE, FRAMES_PER_BUFFER));
        // we won't output out of range samples so don't bother clipping them.
        settings.flags = pa::stream_flags::CLIP_OFF;
        
        let (tx, rx)  = mpsc::channel();
        self.tx = tx;

        let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
            let mut idx = 0;
            for _ in 0..frames {
                buffer[idx] = rx.recv().unwrap();
                println!("send pa: {}",buffer[idx]);
                idx += 1;
            }
            pa::Continue
        };

        let mut stream = try!(pa.open_non_blocking_stream(settings, callback));

        try!(stream.start());
        while(true) {
            self.process();
        }

        try!(stream.stop());
        try!(stream.close());
        Ok(())
    }

    fn process(&mut self) {
        {
            let mut processors = &mut self.processors;
            let topo: &Vec<usize>; 
            match self.topologically_ordered {
                Some(ref x) => topo = x,
                None        => panic!(), //TODO: implement proper error handling!
            }
            for processor_num in topo {
                let processor_num_two = processor_num;
                let processor_num_three = processor_num;
                let mut boxed_processor: &mut Box<Processor> = &mut processors[*processor_num];
                let mut processor = boxed_processor;
                let result: Vec<Signal>;
                {
                    let input : &Vec<Signal> = &self.input_buffers[*processor_num_two];
                    result = processor.process(input);
                }
                let connections: &Vec<Vec<(usize, usize)>> = &self.adjacency_list[*processor_num_three];
                for (plug_num, plug) in connections.iter().enumerate() {
                    for other in plug {
                        let (other_processor_num, other_input) = *other;
                        self.input_buffers[other_processor_num][other_input] = result[plug_num].clone();
                    }
                }
            }
        }
        for (io_slot_num, io_processor_num) in (&self.io).iter().enumerate() {
            let io_processor: &Box<Processor> = &self.processors[*io_processor_num];
            if io_processor.type_name() == "Dac" {
                //TODO: implement properly, only supports one channel
                let signal: f32;
                match self.input_buffers[*io_processor_num][0] {
                    Signal::Sound(a) => signal = a as f32,
                    _                => panic!(),
                }
                println!("sendone: {}",signal);
                self.tx.send(signal);
                break;
            }
        }
    }

    fn connect(self: &mut Mesh, output: (usize, usize), input: (usize, usize)) -> bool {
    // output: (processor, plug), input: (processor, plug)
        self.adjacency_list[output.0][output.1].push((input.0, input.1));
        let connections_match = self.check_types();
        if connections_match {
            self.order_topologically();
            true
        } else {
            self.adjacency_list[output.0][output.1].pop();
            false
        }
    }

    fn order_topologically(self: &mut Mesh) {

        //utility lists
        let mut outgoing_connections: Vec<HashMap<usize, usize>> =
            vec![HashMap::new(); self.adjacency_list.len()];
        let mut incoming_connections: Vec<HashMap<usize, usize>> =
            vec![HashMap::new(); self.adjacency_list.len()];

        // fill up incoming_connections, outgoing connections and
        // amt_inc_connections.
        for (processor_num, processor) in
             self.adjacency_list.iter().enumerate() {
            for out_plug in processor {
                for connection in out_plug {
                    let &(in_processor, _) = connection;
                    incoming_connections[in_processor].insert(processor_num, 0);
                    outgoing_connections[processor_num].insert(in_processor, 0);
                }
            }
        }


        let mut to_visit: LinkedList<usize>     = LinkedList::new();
        let mut  visited: HashMap<usize, usize> = HashMap::new();
        let mut  ordered: Vec<usize>            = Vec::new();
        
        for (processor, connections) in incoming_connections.iter().enumerate() {
            if connections.len() == 0 {
                to_visit.push_front(processor);
            }
        }

        let mut count = 0;

        while visited.len() < self.adjacency_list.len() {
            for (processor, connections) in 
                    incoming_connections.iter().enumerate() {

                if !visited.contains_key(&processor) && connections.len() == 0  {
                    to_visit.push_front(processor);
                }
            }
            
            let current: usize;
            match to_visit.pop_front() {
                Some(x) => current = x,
                None => panic!(),
            }
            let outgoing: &HashMap<usize, usize> = &outgoing_connections[current];
            for (new, _) in outgoing {
                incoming_connections[*new].remove(&current);    
            }

            visited.insert(current, 0);
            ordered.push(current);
            count += 1;
        }
        if count < self.adjacency_list.len() {
            self.topologically_ordered = Option::None;
        } else {
            self.topologically_ordered = Option::Some(ordered);
        }
        
    }

    fn check_types(self: &Mesh) -> bool {
        for (processor_num, processor) in self.adjacency_list.iter().enumerate() {
            for (out_plug_num, out_plug) in processor.iter().enumerate() {
                for in_processor in out_plug {
                    let (in_processor_num, in_plug) = *in_processor;
                    let this_plug  = &self.processors[processor_num]
                        .output_types()[out_plug_num];
                    let other_plug = &self.processors[in_processor_num]
                        .input_types_and_defaults()[in_plug];
                    let return_value: bool;
                    match *this_plug {
                        Signal::Sound(_) => match *other_plug {
                            Signal::Sound(_) => return_value = true,
                            Signal::Int(_)   => return_value = false,
                        },
                        Signal::Int(_)   => match *other_plug {
                            Signal::Sound(_) => return_value = false,
                            Signal::Int(_)   => return_value = true,
                        }
                    }
                    if !return_value {
                        return false;
                    }
                }
            }
        }
        true
    }
}

fn contains(list: &LinkedList<usize>, other: &usize) -> bool {
    for i in list {
        if i == other {
            return true;
        }
    }
    return false;
}

