
#![allow(dead_code)]


use std::option;
use std::vec::Vec;
use std::collections::LinkedList;
use std::collections::HashMap;
use super::super::slope::slope::Slope;

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
    mesh.connect((3, 0), (0, 0));
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
    fn inputs_amt(self: &Self) -> usize;
    fn outputs_amt(self: &Self) -> usize;
}

pub enum Signal {
    Sound(f32)
}

pub struct Mesh {
    processors: Vec<Box<Processor>>,
    //[out_processor][out_plug][connection](in_processor, in_plug)
    adjacency_list: Vec<Vec<Vec<(usize, usize)>>>,
    topologically_ordered: Option<Vec<usize>>,
}

impl Mesh {

    fn new() -> Mesh {
        Mesh {
            processors: Vec::new(),
            adjacency_list: Vec::new(),
            topologically_ordered: Option::Some(Vec::new()),
        }
    }

    fn add_processor<T: 'static + Processor> 
      (self: &mut Mesh, processor: T) -> usize {

        self.adjacency_list.push(Vec::new());
        for _ in 0..processor.outputs_amt() {
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
        self.processors.push(boxed_processor);
        self.adjacency_list.len() - 1
    }

    fn connect(self: &mut Mesh, output: (usize, usize), input: (usize, usize)) {
    // output: (processor, plug), input: (processor, plug)
        self.adjacency_list[output.0][output.1].push((input.0, input.1));
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

