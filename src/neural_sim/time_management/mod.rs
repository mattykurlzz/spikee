use std::cell::Ref;
use std::cell::RefCell;
use std::thread; // todo: use Arc<Mutex<T>> to allow for safe concurrency?
use std::sync::{Arc, Mutex, Condvar};

use crate::neural_sim::{LifNeuron, Neuron};
use std::io::{Error, ErrorKind};

static SIM_DEFINED: bool = false;

pub trait ControllingUnit {
    fn add_to_registry(&mut self, added_subordinate: Arc<Mutex<dyn Neuron>>);
    fn get_len(&self) -> usize;
    fn start_planned(&self);
}

pub struct Director {
    subordinates: Vec<Arc<Mutex<dyn Neuron>>>,
}

impl ControllingUnit for Director {
    fn add_to_registry(&mut self, added_subordinate: Arc<Mutex<dyn Neuron>>) {
        self.subordinates.push(added_subordinate);
    }

    fn get_len(&self) -> usize {
        self.subordinates.len()
    }

    fn start_planned(&self) {
        let mut thread_handles = Vec::new();
        for subord_trait in &self.subordinates {
            let self_copy = Arc::clone(subord_trait); 
            let subord_thread_handle = thread::spawn(move || {
                let mut mutex = self_copy.lock().unwrap();
                mutex.init(0);
        });
            thread_handles.push(subord_thread_handle);
        }

        for handle in thread_handles {
            handle.join().unwrap();
        }
    }
}

impl Director {
    pub fn new() -> Option<Self> {
        Some(Self {
            subordinates: vec![],
        })
        // sim.register_director(dir)
    }
}

pub struct Simulation {
    controlled_directors: Vec<Director>,
    sim_time: u32,
}

impl Simulation {
    pub fn new(sim_time: u32) -> Result<Self, String> {
        if SIM_DEFINED {
            Err("FileAlreadyExistsError: only one Simulation entity can be defined!".to_string())
        } else {
            Ok(Self {
                controlled_directors: Vec::new(),
                sim_time,
            })
        }
    }
    pub fn register_director(&mut self, director: Director) -> Option<&mut Director> {
        self.controlled_directors.push(director);
        self.controlled_directors.last_mut()
    }
    pub fn start(&mut self) {
        for director in &mut self.controlled_directors {
            director.start_planned();
        }
    }
}
