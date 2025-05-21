use std::cell::Ref;
use std::cell::RefCell;
use std::sync::Barrier;
use std::sync::RwLock;
use std::thread; // todo: use Arc<Mutex<T>> to allow for safe concurrency?
use std::sync::{Arc, Mutex, Condvar};

use crate::neural_sim::{LifNeuron, Neuron};
use std::io::{Error, ErrorKind};

static SIM_DEFINED: bool = false;

pub trait ControllingUnit {
    fn add_to_registry(&mut self, added_subordinate: Arc<Mutex<dyn Neuron>>);
    fn get_len(&self) -> usize;
    fn start_planned(&mut self);
    fn increment_time(&mut self);
}

pub struct Director {
    subordinates: Vec<Arc<Mutex<dyn Neuron>>>,
    sim_time: u32,
    cur_time: u32,
}

impl ControllingUnit for Director {
    fn add_to_registry(&mut self, added_subordinate: Arc<Mutex<dyn Neuron>>) {
        self.subordinates.push(added_subordinate);
    }

    fn get_len(&self) -> usize {
        self.subordinates.len()
    }
    
    fn increment_time(&mut self) {
        self.cur_time += 1;
    }

    fn start_planned(&mut self) {
        let mut thread_handles = Vec::new();
        let cur_time_arc = Arc::new(RwLock::new(self.cur_time));
        let sim_time_arc = Arc::new(RwLock::new(self.sim_time));
        let timestep_barrier = Arc::new(Barrier::new(self.subordinates.len() + 1));
        let main_thread_barrier = Arc::clone(&timestep_barrier);
        
        for subord_trait in &self.subordinates {
            let self_copy = Arc::clone(subord_trait); 
            let (cut_time_clone, sim_time_clone) = (Arc::clone(&cur_time_arc), Arc::clone(&sim_time_arc));
            let barrier_clone = Arc::clone(&timestep_barrier);

            let subord_thread_handle = thread::spawn(move || {
                let mut lock = self_copy.lock().unwrap();
                lock.init(0);
                
                barrier_clone.wait(); // wait for all threads in order to sync time shift
                while *cut_time_clone.read().unwrap() != *sim_time_clone.read().unwrap() {
                    println!("a step passed {}, {}", *cut_time_clone.read().unwrap(), *sim_time_clone.read().unwrap());
                    barrier_clone.wait();
                    barrier_clone.wait();
                }
        });
            thread_handles.push(subord_thread_handle);
        }
        
        main_thread_barrier.wait(); // sync before time shift
        while self.cur_time != self.sim_time {
            main_thread_barrier.wait();
            self.increment_time();
            *cur_time_arc.write().unwrap() = self.cur_time;
            main_thread_barrier.wait();
        }

        for handle in thread_handles {
            handle.join().unwrap();
        }
    }
}

impl Director {
    pub fn new(sim_time: u32) -> Option<Self> {
        Some(Self {
            subordinates: vec![],
            sim_time,
            cur_time: 0
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
