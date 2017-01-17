#![allow(dead_code)]


use std::option;
use std::vec::Vec;
use std::collections::LinkedList;
use std::collections::HashMap;
use slope::slope::Slope;
use pipe::pipe::Pipe;
use add::add::Add;

#[test]
fn test_adder() {
    let mut mesh: Mesh = Mesh::new();
    for i in 0..5 {
        mesh.add_processor(Add::new());
    }
    mesh.connect((0, 0), (2, 0));
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
        Signal::Sound(a) => assert!(a == 4.0),
    }

}

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
    }
}

fn print_input_buffers(mesh: &Mesh) {
    for (i, device) in mesh.input_buffers.iter().enumerate() {
        for (j, connection) in device.iter().enumerate() {
            let value: f32;
            match *connection {
                Signal::Sound(a) => value = a,
            }
            println!("{}:{}: {}", i, j, value);
        }
    }

}

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
    //mesh.connect((3, 0), (0, 0));
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
}


pub enum Signal {
    Sound(f32)
}

impl Clone for Signal {
    fn clone(&self) -> Signal {
        match *self {
            Signal::Sound(a) => return Signal::Sound(a),
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
}

impl Mesh {

    fn new() -> Mesh {
        Mesh {
            processors: Vec::new(),
            input_buffers: Vec::new(),
            adjacency_list: Vec::new(),
            topologically_ordered: Option::Some(Vec::new()),
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
        self.processors.push(boxed_processor);
        self.order_topologically();
        self.adjacency_list.len() - 1
    }

    fn process(&mut self) {
        let mut processors = &mut self.processors;
        let topo: &Vec<usize>; 
        match self.topologically_ordered {
            Some(ref x) => topo = x,
            None    => panic!(), //TODO: implement proper error handling!
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

    fn connect(self: &mut Mesh, output: (usize, usize), input: (usize, usize)) {
    // output: (processor, plug), input: (processor, plug)
        self.adjacency_list[output.0][output.1].push((input.0, input.1));
        self.order_topologically();
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
}

fn contains(list: &LinkedList<usize>, other: &usize) -> bool {
    for i in list {
        if i == other {
            return true;
        }
    }
    return false;
}

