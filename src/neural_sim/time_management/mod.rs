use std::sync::Barrier;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use std::thread; // todo: use Arc<Mutex<T>> to allow for safe concurrency?

use crate::neural_sim::Neuron;

static SIM_DEFINED: bool = false;

pub trait ControllingUnit {
    fn add_to_registry(&mut self, added_subordinate: Arc<Mutex<dyn Neuron>>);
    fn start_planned(&mut self);
    fn increment_time(&mut self);
    fn spawn_neuron_thread_closure(
        neuron_copy: Arc<Mutex<dyn Neuron>>,
        cur_time_clone: Arc<RwLock<u32>>,
        sim_time_clone: Arc<RwLock<u32>>,
        barrier_clone: Arc<Barrier>,
    ) -> impl Fn();
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

    fn increment_time(&mut self) {
        self.cur_time += 1;
    }

    fn spawn_neuron_thread_closure(
        neuron_copy: Arc<Mutex<dyn Neuron>>,
        cur_time_clone: Arc<RwLock<u32>>,
        sim_time_clone: Arc<RwLock<u32>>,
        barrier_clone: Arc<Barrier>,
    ) -> impl Fn() {
        move || {
            let mut lock = neuron_copy.lock().unwrap();
            lock.init(0);

            let mut cur_time = *cur_time_clone.read().unwrap();
            while cur_time != *sim_time_clone.read().unwrap() {
                println!(
                    "a step passed {}, {}",
                    cur_time,
                    *sim_time_clone.read().unwrap()
                );
                barrier_clone.wait(); // sync before time increment
                while lock.get_earliest_event_available().unwrap() {
                    if *lock.get_earliest_event().unwrap() == cur_time {
                        // ToDo: scan routing table and emmit signal
                        lock.pop_earliest_event();
                    } else {
                        break;
                    }
                }
                barrier_clone.wait(); // sync after time increment
                cur_time = *cur_time_clone.read().unwrap();
            }
            println!("exited process")
        }
    }

    fn start_planned(&mut self) {
        let mut thread_handles = Vec::new();
        let (cur_time_arc, sim_time_arc) = (
            Arc::new(RwLock::new(self.cur_time)),
            Arc::new(RwLock::new(self.sim_time)),
        );
        let timestep_barrier = Arc::new(Barrier::new(self.subordinates.len() + 1));

        for subord_trait in &self.subordinates {
            let self_copy = Arc::clone(subord_trait);
            let (cur_time_clone, sim_time_clone) =
                (Arc::clone(&cur_time_arc), Arc::clone(&sim_time_arc));
            let barrier_clone = Arc::clone(&timestep_barrier);
            let thread_closure = Self::spawn_neuron_thread_closure( self_copy, cur_time_clone, sim_time_clone, barrier_clone);

            let subord_thread_handle = thread::spawn(thread_closure);
            thread_handles.push(subord_thread_handle);
        }

        {
            let main_thread_barrier = Arc::clone(&timestep_barrier);
            while self.cur_time != self.sim_time {
                main_thread_barrier.wait();
                self.increment_time();
                *cur_time_arc.write().unwrap() = self.cur_time;
                main_thread_barrier.wait();
            }
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
            cur_time: 0,
        })
        // sim.register_director(dir)
    }
}

pub struct Simulation {
    controlled_directors: Vec<Director>,
}

impl Simulation {
    pub fn new() -> Result<Self, String> {
        if SIM_DEFINED {
            Err("FileAlreadyExistsError: only one Simulation entity can be defined!".to_string())
        } else {
            Ok(Self {
                controlled_directors: Vec::new(),
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
