use std::collections::btree_map::Range;
use std::sync::mpsc::Sender;
use std::sync::Barrier;
use std::sync::RwLock;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;

use neuron::Neuron; // todo: use Arc<Mutex<T>> to allow for safe concurrency?
use synapse::SynapseGroup;

use crate::neural_sim::neuron::CommonlyCreateable;

pub mod neuron;
pub mod synapse;  

static SIM_DEFINED: bool = false;

type NeuronUniqueId = u32;

pub trait ControllingUnit {
    fn add_to_registry(&mut self, added_subordinate: Arc<Mutex<dyn Neuron>>) -> Option<NeuronUniqueId>;

    fn start_planned(&mut self);
    fn increment_time(&mut self);
    fn spawn_neuron_thread_closure(
        neuron_copy: Arc<Mutex<dyn Neuron>>,
        cur_time_clone: Arc<RwLock<u32>>,
        sim_time_clone: Arc<RwLock<u32>>,
        barrier_clone: Arc<Barrier>,
        tx: Sender<u32>,
    ) -> impl Fn();
    fn create_link(&mut self, source: NeuronUniqueId, destination: NeuronUniqueId, weight: f32);
}

struct NeuronIdWeightPair{
    id: NeuronUniqueId,
    weight: f32
}

type ForwardOneToManyConnection  = Vec<NeuronIdWeightPair>;

struct NeuronRegistrator {
    next_available_id: NeuronUniqueId,
    assigned_id_vec: Vec<NeuronUniqueId>,
    connection_map: HashMap<NeuronUniqueId, ForwardOneToManyConnection>,
}

impl NeuronRegistrator {
    fn book_id(&mut self) -> Option<NeuronUniqueId> {
        // todo: book id's, create pairs of id-recepient_function, pass ids to synapses, call recepient_functions whenever neuron fires
        let ret = self.next_available_id;
        self.assigned_id_vec.push(self.next_available_id);
        self.next_available_id += 1; 
        Some(ret)
    }
    fn new() -> Self {
        Self { next_available_id: 0, assigned_id_vec: Vec::new(), connection_map: HashMap::new() }
    }

    fn fire_from_id(&mut self, caller_id: NeuronUniqueId, map_ref: &mut HashMap<NeuronUniqueId, Arc<Mutex<dyn Neuron>>>, time_step: u32) {
        println!("firing from {caller_id}!");
        let reicevers_list: &mut ForwardOneToManyConnection  = match self.connection_map.get_mut(&caller_id) {
            Some(val) => val,
            None => &mut Vec::<NeuronIdWeightPair>::new()
        };
        for recvr_id_weight_pair in reicevers_list {
            let recv_id = recvr_id_weight_pair.id;
            let synapse_weight = recvr_id_weight_pair.weight;
            let mux_clone = map_ref.get_mut(&recv_id).unwrap().clone();
            mux_clone.lock().unwrap().recieve_signal(time_step, synapse_weight);
        }
    }

    fn link(&mut self, source_id: NeuronUniqueId, dest_id: NeuronUniqueId, weight: f32) {
        let added_pair = NeuronIdWeightPair { id: dest_id, weight};
        if let Vacant(e) = self.connection_map.entry(source_id) {
            e.insert(vec![added_pair]);
        } else {
            self.connection_map.get_mut(&source_id).unwrap().push(added_pair);
        }       
    }
}

pub struct Director {
    subordinates: Vec<Arc<Mutex<dyn Neuron>>>,
    sim_time: u32,
    cur_time: u32,
    planner: NeuronRegistrator,
    id_to_mux_map: HashMap<NeuronUniqueId, Arc<Mutex<dyn Neuron>>>,
}

impl ControllingUnit for Director {
    fn add_to_registry(&mut self, added_subordinate: Arc<Mutex<dyn Neuron>>) -> Option<NeuronUniqueId> {
        let id = self.planner.book_id().unwrap();

        let trait_clone = Arc::clone(&added_subordinate);
        thread::spawn(move || {
            let mut trait_clone_lock = trait_clone.lock().unwrap();
            trait_clone_lock.set_id(id);
        }).join().unwrap();

        self.id_to_mux_map.insert(id, Arc::clone(&added_subordinate));
        self.subordinates.push(added_subordinate);
        Some(id)
    }

    fn increment_time(&mut self) {
        self.cur_time += 1;
    }

    fn spawn_neuron_thread_closure(
        neuron_copy: Arc<Mutex<dyn Neuron>>,
        cur_time_clone: Arc<RwLock<u32>>,
        sim_time_clone: Arc<RwLock<u32>>,
        barrier_clone: Arc<Barrier>,
        tx: Sender<u32>,
    ) -> impl Fn() {
        move || {
            {
                let mut lock = neuron_copy.lock().unwrap();
                lock.init();
            }
            let mut cur_time = *cur_time_clone.read().unwrap();

            loop { // main neuron loop
                barrier_clone.wait(); // sync before time increment
                {
                    let mut lock = neuron_copy.lock().unwrap();
                    /* in this interval, neurons compute, fire, receive signals */
                    while lock.get_earliest_event_available().unwrap() {
                        println!("checking at time: {}", cur_time);
                        if *lock.get_earliest_event().unwrap() == cur_time {
                            let fired_id = lock.fire().unwrap();
                            lock.pop_earliest_event();
                            
                            tx.send(fired_id).unwrap();
                        } else {
                            break;
                        }
                    }
                }
                barrier_clone.wait(); // sync before time increment
                barrier_clone.wait(); // sync after time increment
                cur_time = *cur_time_clone.read().unwrap();
            }
        }
    }

    fn start_planned(&mut self) {
        let mut thread_handles = Vec::new();
        let (cur_time_arc, sim_time_arc) = (
            Arc::new(RwLock::new(self.cur_time)),
            Arc::new(RwLock::new(self.sim_time)),
        );
        let timestep_barrier = Arc::new(Barrier::new(self.subordinates.len() + 1));
        let (tx, rx) = mpsc::channel::<u32>();

        for subord_trait in &self.subordinates {
            let self_copy = Arc::clone(subord_trait);
            let (cur_time_clone, sim_time_clone) =
                (Arc::clone(&cur_time_arc), Arc::clone(&sim_time_arc));
            let barrier_clone = Arc::clone(&timestep_barrier);

            let thread_closure = Self::spawn_neuron_thread_closure( self_copy, cur_time_clone, sim_time_clone, barrier_clone, tx.clone());

            let subord_thread_handle = thread::spawn(thread_closure);
            thread_handles.push(subord_thread_handle);
        }

        {
            let main_thread_barrier = Arc::clone(&timestep_barrier);
            while self.cur_time != self.sim_time {
                let mut none_neurons_have_fired: bool = true;

                main_thread_barrier.wait();
                main_thread_barrier.wait();

                /* after this, all neurons await barrier in new inputs and do not hold lock */
                for sender_id in rx.try_iter() {
                    none_neurons_have_fired = false;

                    println!("emmit request got from {sender_id}");
                    self.planner.fire_from_id(sender_id, &mut self.id_to_mux_map, self.cur_time);
                }

                if none_neurons_have_fired {
                    println!(
                        "a step {} passed of {}\n\n",
                        self.cur_time,
                        self.sim_time
                    );

                    self.increment_time();
                    *cur_time_arc.write().unwrap() = self.cur_time;
                }

                main_thread_barrier.wait();
            }
        }
        // ** join is unnecessary as simulation is already checkpointed **
        // for handle in thread_handles {
        //     handle.join().unwrap();
        // }
    }
    
    fn create_link(&mut self, source: NeuronUniqueId, destination: NeuronUniqueId, weight: f32) {
        // self.tmp_source_dest_pairs.push([source, destination]);
        self.planner.link(source, destination, weight);
    }
}

impl Director {
    pub fn new(sim_time: u32) -> Option<Self> {
        Some(Self {
            subordinates: vec![],
            sim_time,
            cur_time: 0,
            planner: NeuronRegistrator::new(),
            id_to_mux_map: HashMap::new(),
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
