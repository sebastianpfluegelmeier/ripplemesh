use pipe::pipe::Pipe;
use add::add::Add;
use mult::mult::Mult;
use intpipe::intpipe::Intpipe;
use dac::dac::Dac;
use sine::sine::Sine;
use std::vec::Vec;
use mesh::mesh::Signal;
use mesh::mesh::Mesh;
use mesh::mesh::Processor;

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

// #[test]
fn dac_test() {
    let mut mesh: Mesh = Mesh::new();
    mesh.add_processor(Sine::new());
    mesh.add_processor(Sine::new());
    mesh.input_buffers[1][0] = Signal::Sound(0.5);
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

// need proper refactoring before it can be used again
/*
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
*/


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
        mesh.add_processor(Intpipe::new());
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

#[test]
fn io_test() {
    let mut mesh: Mesh = Mesh::new();
    mesh.run();
}
