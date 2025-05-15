use crate::neural_sim::LifNeuron;
use std::io::{Error, ErrorKind};

static SIM_DEFINED: bool = false;

pub trait ControllingUnit {
    fn add_to_registry(&mut self, added_subordinate: LifNeuron);
    fn get_len(&self) -> usize;
}

#[derive(Debug)]
pub struct Director {
    subordinates: Vec<LifNeuron>,
}

impl ControllingUnit for Director {
    fn add_to_registry(&mut self, added_subordinate: LifNeuron){
        self.subordinates.push(added_subordinate);
    }

    fn get_len(&self) -> usize {
        self.subordinates.len()
    }
}

impl Director {
    pub fn new(sim: &mut Simulation) -> Option<&Self> {
        let dir = Self {
            subordinates: vec![],
        };
        sim.register_director(dir)
    }
}

#[derive(Debug)]
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
    fn register_director(&mut self, director: Director) -> Option<&Director> {
        self.controlled_directors.push(director);
        self.controlled_directors.last()
    }
}
