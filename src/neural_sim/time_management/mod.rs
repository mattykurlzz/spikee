pub mod event_schedule_entry;

use crate::neural_sim::LifNeuron;
use event_schedule_entry::{Callable, EventEntry};
use std::io::{Error, ErrorKind};

static SIM_DEFINED: bool = false;

pub trait ControllingUnit<F>
where
    F: FnOnce(i32),
{
    fn add_to_registry<'a>(&'a mut self, added_subordinate: LifNeuron) -> Option<&'a mut LifNeuron>;
    fn get_len(&self) -> usize;
    fn start_planned(&mut self);
    fn add_event(&mut self, event: EventEntry<F>);
}

pub struct Director<F>
where
    F: FnOnce(i32),
{
    subordinates: Vec<LifNeuron>,
    events_queue: Vec<EventEntry<F>>,
}

impl<F> ControllingUnit<F> for Director<F>
where
    F: FnOnce(i32),
{
    fn add_to_registry<'a>(&'a mut self, added_subordinate: LifNeuron) -> Option<&'a mut LifNeuron> {
        self.subordinates.push(added_subordinate);
        self.subordinates.last_mut()
    }

    fn get_len(&self) -> usize {
        self.subordinates.len()
    }

    fn start_planned(&mut self) {
        // for event_entry in self.events_queue {
        //     event_entry.call();
        // }
    }

    fn add_event(&mut self, event: EventEntry<F>) {
        self.events_queue.push(event);
    }
}

impl<F> Director<F> 
where F: FnOnce(i32){
    pub fn new() -> Option<Self> {
        Some(Self {
            subordinates: vec![],
            events_queue: vec![],
        })
        // sim.register_director(dir)
    }
}

pub struct Simulation<F>
where F: FnOnce(i32) {
    controlled_directors: Vec<Director<F>>,
    sim_time: u32,
}

impl<F> Simulation<F>
where F: FnOnce(i32)
{
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
    pub fn register_director(&mut self, director: Director<F>) -> Option<&mut Director<F>> {
        self.controlled_directors.push(director);
        self.controlled_directors.last_mut()
    }
    pub fn start(&mut self) {
        for director in &mut self.controlled_directors {
            director.start_planned();
        }
    }
}
