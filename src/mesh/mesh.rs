#![allow(dead_code)]

extern crate portaudio;

use std::io::{self, Read};
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
use engine::engine::Engine;
use engine::engine::CallbackMessage;
use dummy::dummy::Dummy;
use constant::constant::Constant;

use self::portaudio as pa;

pub const SAMPLERATE: f64 = 44100.0;
pub const CHANNELS: i32 = 1;
pub const FRAMES_PER_BUFFER: u32 = 64;

type AdjList = Vec<Vec<Vec<(usize, usize)>>>;

fn adj_clone(input: &AdjList) -> AdjList {
    let mut clone: AdjList = Vec::new();
    for i in input {
        let mut i_vec = Vec::new();
        for j in i {
            let mut j_vec = Vec::new();
            for k in j {
                j_vec.push(((*k).0, (*k).1));
            }
            i_vec.push(j_vec);
        }
        clone.push(i_vec);
    }
    clone
}

pub type TopoList = Option<Vec<usize>>;

fn topo_clone(input: &TopoList) -> TopoList {
    match input {
        &Option::Some(ref a) => {
            let mut clone: Vec<usize> = Vec::new();
            for i in a {
                clone.push(*i);
            }
            return Option::Some(clone);
        },
        &Option::None    => return Option::None,
    }
}

pub type IoList = Vec<usize>;

fn io_clone(input: &IoList) -> IoList {
    let mut out = Vec::new();
    for i in input {
        out.push(*i);
    }
    out
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
    pub processor_types: Vec<(Vec<Signal>, Vec<Signal>, String)>,
    pub input_buffers: Vec<Vec<Signal>>, // computed signals are stored here until
                                     //they get processed.
    //[out_processor][out_plug][connection](in_processor, in_plug)
    adjacency_list: AdjList,
    pub topologically_ordered: Option<Vec<usize>>,
    ios: Vec<usize>,
    tx: Sender<f32>,
}

impl Mesh {

    pub fn new() -> Mesh {
        let (ttx, _) = mpsc::channel();
        Mesh {
            processor_types: Vec::new(),
            input_buffers: Vec::new(),
            adjacency_list: Vec::new(),
            tx: ttx,
            topologically_ordered: Option::Some(Vec::new()),
            ios: Vec::new(),
        }
    }

    pub fn add_processor<T: 'static + Processor> 
      (self: &mut Mesh, processor: T) -> Box<Processor> {

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
            self.ios.push(self.adjacency_list.len() - 1);
        }
        self.processor_types.push(((*boxed_processor).input_types_and_defaults(),
                                   (*boxed_processor).output_types(),
                                   (*boxed_processor).type_name()));
        self.order_topologically();
        boxed_processor
    }

    pub fn run(&mut self) -> Result<(), pa::Error> {
        let pa = try!(pa::PortAudio::new());
        let (tx, rx): (mpsc::Sender<CallbackMessage>, mpsc::Receiver<CallbackMessage>) = mpsc::channel();
        let mut engine: Engine =  Engine::new(rx);

        let mut settings = 
            try!(pa.default_output_stream_settings(
                    CHANNELS, SAMPLERATE, FRAMES_PER_BUFFER));
        // we won't output out of range samples so don't bother clipping them.
        settings.flags = pa::stream_flags::CLIP_OFF;
        

        let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
            let mut idx = 0;
            for _ in 0..frames {
                buffer[idx] = engine.process()[0];
                idx += 1;
            }
            pa::Continue
        };

        let mut stream = try!(pa.open_non_blocking_stream(settings, callback));

        try!(stream.start());
        while true {
            //Just temporary for debugging.
            tx.send(self.prompt());
        }

        try!(stream.stop());
        try!(stream.close());
        Ok(())
    }

    pub fn prompt(&mut self) -> CallbackMessage {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
	    Result::Ok(_)  => (),
	    Result::Err(_) => {
		println!("could not read line");
	        panic!("could not read line");  
	    },
	}
        let inputs: Vec<&str> = input.split(' ').collect();
        let mut message: CallbackMessage = CallbackMessage::Processor(Box::new(Dummy::new()));
        match inputs[0] {
            "new" => {
                match inputs[1].trim_right() {
                    "constant" => message = CallbackMessage::Processor(self.add_processor(Constant::new())),
                    "sine" => message = CallbackMessage::Processor(self.add_processor(Sine::new())),
                    "add"  => message = CallbackMessage::Processor(self.add_processor(Add::new())),
                    "mult" => message = CallbackMessage::Processor(self.add_processor(Mult::new())),
                    "dac"  => message = CallbackMessage::Processor(self.add_processor(Dac::new())),
                    x      => println!("module \"{}\" not known", x),
                }
            },
            "connect" => {
                let c1 = inputs[1].parse::<usize>().unwrap();
                let c2 = inputs[2].parse::<usize>().unwrap();
                let c3 = inputs[3].parse::<usize>().unwrap();
                let c4 = inputs[4].trim_right().parse::<usize>().unwrap();
                if self.connect((c1, c2), (c3, c4)) { message = 
                    CallbackMessage::Connections((adj_clone(&self.adjacency_list),
                    topo_clone(&self.topologically_ordered).unwrap_or(Vec::new()/*really shitty way of handling error*/),
                    io_clone(&self.ios)));
                } else { println!("types dont match");} 
            },
            "constant" => {
                message = CallbackMessage::Constant((*inputs[1]).parse().unwrap(), (*inputs[2]).parse().unwrap())
            },
            _ => println!("command not found"),
        }
        message
    }

    pub fn connect(self: &mut Mesh, output: (usize, usize), input: (usize, usize)) -> bool {
    // output: (processor, plug), input: (processor, plug)
	match self.topologically_ordered {
	    Option::Some(_) => (),
	    Option::None => return false,
	}
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

    pub fn order_topologically(self: &mut Mesh) {

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
                    let this_plug  = &self.processor_types[processor_num]
                        .1[out_plug_num];
                    let other_plug = &self.processor_types[in_processor_num]
                        .0[in_plug];
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

