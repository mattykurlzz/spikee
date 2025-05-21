use std::{sync::{Arc, Mutex, Condvar}, thread};
use time_management::ControllingUnit;

mod neuron_state{
    #[derive(Debug)]
    pub struct NeuronState {
        spike: bool,
        waiting_next_time_step: bool,
    }

    impl NeuronState {
        pub fn new() -> Option<Self> {
            Some(Self {
                spike: false,
                waiting_next_time_step: false,
            })
        }
        fn null_every(&mut self) {
            self.spike = false;
        }
        pub fn set_spike_exclusive(&mut self, state: bool) {
            self.null_every();
            self.spike = state;
            self.waiting_next_time_step = true;
        }
        pub fn get_blocked(&mut self) -> bool {
            self.waiting_next_time_step
        }
        pub fn get_spike(&mut self) -> bool {
            self.spike
        }
    }
}

use neuron_state::NeuronState;

pub mod time_management;

pub trait TimeDependent{
    fn register(self, director: &mut time_management::Director);
}

pub trait Neuron: Send + Sync {
    fn init(&mut self, time_step: i32);
    fn recieve_signal(&mut self, time_step: i32, signal: f32);
    fn emmit_signal(&mut self);
}

#[derive(Debug)]
pub struct LifNeuron {
    current_potential: f32,
    leak_rate: f32,
    state: NeuronState,
    spikes_queue: Vec<i32>,
}

impl LifNeuron {
    pub fn new(leak: f32) -> Self {
        Self {
            current_potential: 0.,
            leak_rate: leak,
            state: NeuronState::new().unwrap(),
            spikes_queue: Vec::new(),
        }
    }
}

impl Neuron for LifNeuron {
    fn init(&mut self, time_step: i32) {
        println!("Called init!. My leak is {}", self.leak_rate);
        self.emmit_signal();
    }
    fn recieve_signal(&mut self, time_step: i32, signal: f32) {
        println!("Called recv_sig");
    }
    fn emmit_signal(&mut self) {
        println!("Called emmit");
    }
}
// impl Leaky for LifNeuron {
//     fn perform_step_leak(mut self) -> () {
//         self.current_potential -= self.leak_rate
//     }
// }

impl TimeDependent for LifNeuron{
    fn register(self, director: &mut time_management::Director) {
        let passed_trait: Arc<Mutex<dyn Neuron>> = Arc::new(Mutex::new(self));
        director.add_to_registry(passed_trait); // todo: add meaningfull error handling
    }
}
