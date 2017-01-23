extern crate jack;
use mesh::mesh::Signal;
use mesh::mesh::Processor;
use std::io;
use std::str::FromStr;
use std::sync::mpsc::channel;
use jack::prelude::{AudioOutPort, AudioOutSpec, Client, JackClient, JackControl, ProcessHandler,
                    ProcessScope, client_options};

pub struct Jack;

fn read_freq() -> Option<f64> {
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => u16::from_str(&user_input.trim()).ok().map(|n| n as f64),
        Err(_) => None,
    }
}

impl Processor for Jack {
    fn process(&mut self, input: &Vec<Signal>) -> Vec<Signal> {

        let (mut client, _status) = Client::open("rust_jack_sine", client_options::NO_START_SERVER)
            .unwrap();

        let mut out_port = client.register_port("sine_out", AudioOutSpec::default()).unwrap();
        let mut frequency = 220.0;
        let sample_rate = client.sample_rate();
        let frame_t = 1.0 / sample_rate as f64;
        let mut time = 0.0;
        let (tx, rx) = channel();

        let process = ProcessHandler::new(move |ps: &ProcessScope| -> JackControl {
            // Get output buffer
            let mut out_p = AudioOutPort::new(&mut out_port, ps);
            let out: &mut [f32] = &mut out_p;

            // Check frequency requests
            while let Ok(f) = rx.try_recv() {
                time = 0.0;
                frequency = f;
            }

            // Write output
            for v in out.iter_mut() {
                let x = frequency * time * 2.0 * std::f64::consts::PI;
                let y = x.sin();
                *v = y as f32;
                time += frame_t;
            }

            // Continue as normal
            JackControl::Continue
        });

        //
        let active_client = client.activate(process).unwrap();

        println!("Enter an integer value to change the frequency of the sine wave.");
        while let Some(f) = read_freq() {
            tx.send(f).unwrap();
        }

        active_client.deactivate().unwrap();
        vec![]
    }

    fn input_types_and_defaults(&self) -> Vec<Signal> {
        vec![Signal::Sound(0.0)]
    }

    fn output_types(&self) -> Vec<Signal> {
        vec![]
    }
}

impl Jack {
    pub fn new() -> Jack {
        Jack
    }
}
