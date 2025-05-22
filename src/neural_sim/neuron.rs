use std::{sync::{Arc, Mutex}};

use super::{synapse::Connection, Director, NeuronUniqueId};
use super::ControllingUnit;
use super::synapse::SynapseGroup;

/* mod neuron_state{
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
            self.spike = state; self.waiting_next_time_step = true;
        }
        pub fn get_blocked(&mut self) -> bool {
            self.waiting_next_time_step
        }
        pub fn get_spike(&mut self) -> bool {
            self.spike
        }
    }
} */

// use neuron_state::NeuronState;

pub trait TimeDependent{
    fn register(self, director: &mut Director) -> NeuronUniqueId;
}

pub trait Neuron: Send + Sync {
    fn init(&mut self, time_step: u32);
    // fn recieve_signal(&mut self, time_step: u32, signal: f32);
    fn emmit_signal(&mut self, time_step: u32);
    fn get_earliest_event(&self) -> Option<&u32>;
    fn get_earliest_event_available(&self) -> Option<bool>;
    fn pop_earliest_event(&mut self);
    fn fire(&self) -> &Vec<Connection>; // todo: funciton must be implemented but shouldn't be changed by user
}

#[derive(Debug)]
pub struct LifNeuron {
    // current_potential: f32,
    leak_rate: f32,
    // state: NeuronState,
    spikes_queue: Vec<u32>,
    connections: SynapseGroup,
}

impl LifNeuron {
    pub fn new(leak: f32) -> Self {
        Self {
            // current_potential: 0.,
            leak_rate: leak,
            // state: NeuronState::new().unwrap(),
            spikes_queue: Vec::new(),
            connections: SynapseGroup::new().unwrap(),
        }
    }
    pub fn add_events_entry(&mut self, step: u32) {
        self.spikes_queue.push(step);
        self.spikes_queue.sort();
    }
    pub fn get_earliest_event_int(&self) -> Option<&u32> {
        self.spikes_queue.first()
    }
    pub fn pop_earliest_event_int(&mut self){
        self.spikes_queue.remove(0);
    }
}

impl Neuron for LifNeuron {
    fn init(&mut self, time_step: u32) {
        println!("Called init!. My leak is {}", self.leak_rate);
        self.emmit_signal(time_step);
    }
    // fn recieve_signal(&mut self, time_step: u32, signal: f32) {
    //     println!("Called recv_sig");
    // }
    fn emmit_signal(&mut self, time_step: u32) {
        println!("Called emmit registrator");
        self.add_events_entry(time_step);
    }
    fn get_earliest_event(&self) -> Option<&u32> {
        self.get_earliest_event_int()
    }
    fn pop_earliest_event(&mut self) {
        self.pop_earliest_event_int();
    }
    fn get_earliest_event_available(&self) -> Option<bool> {
        match self.get_earliest_event_int() {
            Some(_) => Some(true), 
            None => Some(false)
        }
    }
    fn fire(&self) -> &Vec<Connection> {
        self.connections.fire()
    }
}
// impl Leaky for LifNeuron {
//     fn perform_step_leak(mut self) -> () {
//         self.current_potential -= self.leak_rate
//     }
// }

impl TimeDependent for LifNeuron{
    fn register(self, director: &mut Director) -> NeuronUniqueId {
        let passed_trait: Arc<Mutex<dyn Neuron>> = Arc::new(Mutex::new(self));
        director.add_to_registry(passed_trait); // todo: add meaningfull error handling
    }
}
