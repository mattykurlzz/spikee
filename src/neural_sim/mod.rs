use std::collections::{HashMap, hash_map::Entry::Vacant};
// use std::error::Error;
use std::fs::File;
use std::sync::{Arc, Barrier, Mutex, MutexGuard, RwLock, mpsc, mpsc::Receiver, mpsc::Sender};
use std::thread::{self};

use neuron::Neuron;
use vcd_ng::{IdCode, TimescaleUnit, Writer};
use error::Error;

pub mod neuron;
pub mod error;

type NeuronUniqueId = u32;
type SharedWriter = Arc<Mutex<Writer<File>>>;

#[allow(dead_code)] // Is not used in tests, but should be. //todo
pub enum VecOrValueFloat {
    Vec(Vec<Vec<f32>>),
    Val(f32),
}

#[allow(dead_code)] // Is not used in tests, but should be. //todo
pub enum BatchLinkingRule {
    None,
    FullyConnected,
    UserDefined(fn(usize, usize) -> bool),
}

pub trait ControllingUnit {
    fn add_to_registry(
        &mut self,
        added_subordinate: Arc<Mutex<dyn Neuron>>,
    ) -> Result<NeuronUniqueId, Error>;

    fn init_planned(&mut self, writer_ref: Option<SharedWriter>) -> Result<(), Error>;
    fn start_planned(&mut self) -> Result<(), Error>;
    fn increment_time(&mut self);
    fn spawn_neuron_thread_closure(
        neuron_copy: Arc<Mutex<dyn Neuron>>,
        cur_time_clone: Arc<RwLock<u32>>,
        barrier_clone: Arc<Barrier>,
        tx: Sender<u32>,
        writer: Option<SharedWriter>,
        wire: Option<IdCode>,
    ) -> Result<impl Fn(), Error>;
    fn create_link(&mut self, source: NeuronUniqueId, destination: NeuronUniqueId, weight: f32) -> Result<(), Error>;
    fn create_links_by_rule(
        &mut self,
        sources: &[u32],
        destinations: &[u32],
        weights: VecOrValueFloat,
        rule: BatchLinkingRule,
    ) -> Result<(), Error>;
}

struct NeuronIdWeightPair {
    id: NeuronUniqueId,
    weight: f32,
}

type ForwardOneToManyConnection = Vec<NeuronIdWeightPair>;

struct NeuronRegistrator {
    next_available_id: NeuronUniqueId,
    assigned_id_vec: Vec<NeuronUniqueId>,
    connection_map: HashMap<NeuronUniqueId, ForwardOneToManyConnection>,
}

impl NeuronRegistrator {
    fn book_id(&mut self) -> NeuronUniqueId {
        let ret = self.next_available_id;
        self.assigned_id_vec.push(self.next_available_id);
        self.next_available_id += 1;
        ret
    }
    fn new() -> Self {
        Self {
            next_available_id: 0,
            assigned_id_vec: Vec::new(),
            connection_map: HashMap::new(),
        }
    }

    fn fire_from_id(
        &mut self,
        caller_id: NeuronUniqueId,
        map_ref: &mut HashMap<NeuronUniqueId, Arc<Mutex<dyn Neuron>>>,
        time_step: u32,
    ) -> Result<(), Error> {
        println!("firing from {caller_id}!");
        let reicevers_list: &mut ForwardOneToManyConnection =
            match self.connection_map.get_mut(&caller_id) {
                Some(val) => val,
                None => &mut Vec::<NeuronIdWeightPair>::new(),
            };
        for recvr_id_weight_pair in reicevers_list {
            let recv_id = recvr_id_weight_pair.id;
            let synapse_weight = recvr_id_weight_pair.weight;
            if let Some(mux) = map_ref.get_mut(&recv_id) {
                mux.clone().lock()?.recieve_signal(time_step, synapse_weight);
            };
        }
        Ok(())
    }

    fn link(&mut self, source_id: NeuronUniqueId, dest_id: NeuronUniqueId, weight: f32) -> Result<(), Error> {
        let added_pair = NeuronIdWeightPair {
            id: dest_id,
            weight,
        };
        if let Vacant(e) = self.connection_map.entry(source_id) {
            e.insert(vec![added_pair]);
        } else if let Some(map_ref) = self.connection_map.get_mut(&source_id){
            map_ref.push(added_pair);
        } else {
            return Err(Error::LinkCreate("w"));
        }
        Ok(())
    }
}

pub struct Director {
    subordinates: Vec<Arc<Mutex<dyn Neuron>>>,
    sim_time: u32,
    cur_time: u32,
    planner: NeuronRegistrator,
    id_to_mux_map: HashMap<NeuronUniqueId, Arc<Mutex<dyn Neuron>>>,
    // id: u32,
    name: String,
    rx: Option<Receiver<u32>>,
    main_thread_barrier: Option<Arc<Barrier>>,
    cur_time_arc: Option<Arc<RwLock<u32>>>,
    writer_ref: Option<SharedWriter>,
}

impl ControllingUnit for Director {
    fn add_to_registry(
        &mut self,
        added_subordinate: Arc<Mutex<dyn Neuron>>,
    ) -> Result<NeuronUniqueId, Error> {
        let id = self.planner.book_id();
        let trait_clone = Arc::clone(&added_subordinate);

        match thread::spawn(move || {
            let mut trait_clone_lock = trait_clone.lock().unwrap();
            trait_clone_lock.set_id(id);
        }).join() {
            Ok(()) => {},
            Err(_) => {return Err(Error::JoinHandle);}
        };

        self.id_to_mux_map
            .insert(id, Arc::clone(&added_subordinate));
        self.subordinates.push(added_subordinate);
        Ok(id)
    }

    fn increment_time(&mut self) {
        self.cur_time += 1;
    }

    fn spawn_neuron_thread_closure(
        neuron_copy: Arc<Mutex<dyn Neuron>>,
        cur_time_clone: Arc<RwLock<u32>>,
        barrier_clone: Arc<Barrier>,
        tx: Sender<u32>,
        writer: Option<SharedWriter>,
        wire: Option<IdCode>,
    ) -> Result<impl Fn(), Error> {
        /* 
            As panicking threads pass their panic args to join.handle(), I've decided it's ideomatic to use unwrap()
         */
        let closure = move || {
            let write_cur_signal =
                |wr: &Option<SharedWriter>,
                 wi: &Option<IdCode>,
                 lock: &mut MutexGuard<dyn Neuron + 'static>
                | -> Result<(), Error> {
                    if let Some(v) = wr {
                        v.lock()?.change_real(wi.unwrap(), lock.get_signal().into())?; // See comment above
                    }
                    println!("{}", lock.get_signal());
                    Ok(())
                };

            {
                let mut lock = neuron_copy.lock().unwrap(); // See comment above
                lock.init();
            }

            let mut cur_time = *cur_time_clone.read().unwrap(); // See comment above
            barrier_clone.wait(); // sync with blocked threads to upscope at a right moment
            barrier_clone.wait(); // sync after upscope to define default values            
            {
                let mut lock = neuron_copy.lock().unwrap(); // See comment above
                write_cur_signal(&writer, &wire, &mut lock).unwrap(); // See comment above
            }
            barrier_clone.wait(); // sync to start writing trace
            barrier_clone.wait(); // sync before any actions

            loop {
                // main neuron loop
                barrier_clone.wait(); // sync before concurrent execution
                {
                    let mut lock = neuron_copy.lock().unwrap(); // See comment above

                    lock.perform_leak(cur_time);
                    write_cur_signal(&writer, &wire, &mut lock).unwrap(); // See comment above
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
                barrier_clone.wait(); // sync after time increment to sync current time
                cur_time = *cur_time_clone.read().unwrap();
            }
        };
        Ok(closure)
    }

    fn init_planned(&mut self, writer_ref: Option<SharedWriter>) -> Result<(), Error> {
        let mut thread_handles = Vec::new();
        let cur_time_arc = Arc::new(RwLock::new(self.cur_time));
        let timestep_barrier = Arc::new(Barrier::new(self.subordinates.len() + 1));
        let (tx, rx) = mpsc::channel::<u32>();

        if let Some(ref writer) = writer_ref {
            writer.lock()?.add_module(&self.name)?;
        }

        for subord_trait in &self.subordinates {
            let self_copy = Arc::clone(subord_trait);
            let cur_time_clone = Arc::clone(&cur_time_arc);
            let barrier_clone = Arc::clone(&timestep_barrier);

            let wire = match writer_ref {
                Some(ref value) => {
                    let neuron_lock = self_copy.lock()?;
                    let mut writer_lock = value.lock()?;

                    if let Some(id) = neuron_lock.get_id() {
                        let wire = writer_lock
                            .add_var(
                                vcd_ng::VarType::Real,
                                size_of::<f32>().try_into()?,
                                &id.to_string(),
                                None,
                            )?;
                        Some(wire)
                    } else {None}
                }
                None => None,
            };

            let thread_closure = Self::spawn_neuron_thread_closure(
                self_copy,
                cur_time_clone,
                barrier_clone,
                tx.clone(),
                writer_ref.as_ref().map(Arc::clone),
                wire,
            )?;

            let subord_thread_handle = thread::spawn(thread_closure);
            thread_handles.push(subord_thread_handle);
        }

        timestep_barrier.wait(); // sync with blocked threads to upscope at a right moment

        if let Some(ref writer_mux) = writer_ref {
            let mut lock = writer_mux.lock()?;
            lock.upscope()?; //todo: result
            // let _ = lock.begin(vcd_ng::SimulationCommand::Dumpvars);
        }

        // timestep_barrier.wait(); // sync after upscope, before default vars definition

        // if let Some(ref writer_mux) = writer_ref {
        //     let _ = writer_mux.lock().unwrap().end();
        // }

        self.main_thread_barrier = Some(Arc::clone(&timestep_barrier));
        self.writer_ref = writer_ref;
        self.rx = Some(rx);
        self.cur_time_arc = Some(cur_time_arc);
        
        Ok(())
    }

    fn start_planned(&mut self) -> Result<(), Error>{
        let wait_func = |s: &mut Self| if let Some(barier) = s.main_thread_barrier.as_ref(){
            barier.wait();            
        };

        if let Some(ref writer_mux) = self.writer_ref {
            let mut lock = writer_mux.lock()?;
            let _ = lock.begin(vcd_ng::SimulationCommand::Dumpvars);
        }

        wait_func(self); // sync after upscope, before default vars definition        
        wait_func(self); // sync before any actions
        if let Some(ref writer_mux) = self.writer_ref {
            writer_mux.lock()?.end()?;
        }
        wait_func(self); // sync before any actions
        // self.writer_ref.as_ref().inspect(|v| {if let Ok(mut v) = v.lock() { let _ = v.enddefinitions(); }});

        while self.cur_time != self.sim_time {
            let mut none_neurons_have_fired: bool = true;

            wait_func(self);
            wait_func(self);

            /* after this, all neurons await barrier in new inputs and do not hold lock */
            for sender_id in self.rx.as_mut().unwrap().try_iter() {
                none_neurons_have_fired = false;

                println!("emmit request got from {sender_id}");
                self.planner
                    .fire_from_id(sender_id, &mut self.id_to_mux_map, self.cur_time)?;
            }

            if none_neurons_have_fired {
                println!("a step {} passed of {}\n\n", self.cur_time, self.sim_time);

                self.increment_time();
                *self.cur_time_arc.as_ref().unwrap().write().unwrap() = self.cur_time;
                if let Some(ref writer) = self.writer_ref {
                    let _ = writer.lock().unwrap().timestamp(self.cur_time.into());
                }
            }

            wait_func(self);
        }
        
        Ok(())
    }

    fn create_link(&mut self, source: NeuronUniqueId, destination: NeuronUniqueId, weight: f32) -> Result<(), Error> {
        // self.tmp_source_dest_pairs.push([source, destination]);
        self.planner.link(source, destination, weight)?;
        Ok(())
    }

    fn create_links_by_rule(
        &mut self,
        sources: &[u32],
        destinations: &[u32],
        weights: VecOrValueFloat,
        rule: BatchLinkingRule,
    ) -> Result<(), Error> {
        let linking_rule: fn(usize, usize) -> bool = match &rule {
            BatchLinkingRule::None => |_x: usize, _y: usize| true,
            BatchLinkingRule::FullyConnected => |_x: usize, _y: usize| true,
            BatchLinkingRule::UserDefined(func) => *func,
        };

        let weights = match weights {
            VecOrValueFloat::Vec(x) => x,
            VecOrValueFloat::Val(x) => vec![vec![x; destinations.len()]; sources.len()],
        };

        for (i, source_id) in sources.iter().enumerate() {
            for (j, destination_id) in destinations.iter().enumerate() {
                if linking_rule(i, j) {
                    self.create_link(*source_id, *destination_id, weights[i][j])?;
                }
            }
        }
        Ok(())
    }
}

impl Director {
    pub fn new(sim_time: u32, id: u32) -> Option<Self> {
        Some(Self {
            subordinates: vec![],
            sim_time,
            cur_time: 0,
            planner: NeuronRegistrator::new(),
            id_to_mux_map: HashMap::new(),
            // id,
            name: id.to_string(),
            rx: None,
            main_thread_barrier: None,
            cur_time_arc: None,
            writer_ref: None,
        })
        // sim.register_director(dir)
    }
}

pub struct Simulation {
    controlled_directors: Vec<Director>,
    trace_writer: Option<SharedWriter>,
}

impl Simulation {
    pub fn new(trace: bool, tracefile: Option<&str>) -> Result<Self, Error> {
        let tracefile = tracefile.unwrap_or("tracefile.vcd");

        let writer: Option<SharedWriter> = if trace {
            let file = File::create(tracefile)?;
            let mut w = Writer::new(file);

            w.add_module("sim")?;
            w.timescale(1, TimescaleUnit::US)?;

            Some(Arc::new(Mutex::new(w)))
        } else {
            None
        };

        Ok(Self {
            controlled_directors: Vec::new(),
            trace_writer: writer,
        })
    }
    pub fn register_director(&mut self, director: Director) -> Option<&mut Director> {
        self.controlled_directors.push(director);
        self.controlled_directors.last_mut()
    }

    pub fn start(&mut self) -> Result<(), Error> { 
        let mut director_refs: Vec<Arc<Mutex<Director>>> = Vec::new();
        while let Some(director) = self.controlled_directors.pop() {
            let writer_mut_opt: Option<SharedWriter> = self.trace_writer.as_ref().map(Arc::clone);
            let director_reference = Arc::new(Mutex::new(director));
            director_reference.lock()?.init_planned(writer_mut_opt)?;
            director_refs.push(director_reference);
        }
        if let Some(ref val) = self.trace_writer {
            let mut lock = val.lock().unwrap();
            let _ = lock.upscope();
            let _ = lock.enddefinitions();
        };
        for director_ref in &mut director_refs {
            director_ref.lock().unwrap().start_planned()?;
        }
        if let Some(ref val) = self.trace_writer {
            let mut lock = val.lock().unwrap();
            let _ = lock.end();
        };
        for director_ref in director_refs {
            if let Some(lock) = Arc::into_inner(director_ref) {
                let dir = lock.into_inner()?;
                self.controlled_directors.push(dir);
            };
        }
        Ok(())
    }
}
