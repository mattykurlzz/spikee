use super::*;
use crate::ControllingUnit;
use std::sync::{Arc, Mutex};

pub struct LifNeuron {
    threshold: f32,
    current_potential: f32,
    beta: f32,
    last_leak_time: u32,
    spikes_queue: Vec<u32>,
    id: NeuronUniqueId,
    planned_time_steps: Vec<u32>,
}

impl LifNeuron {
    pub fn new(beta: f32) -> Self {
        Self {
            beta,
            spikes_queue: Vec::new(),
            id: 0,
            planned_time_steps: Vec::new(),
            current_potential: 0.,
            last_leak_time: 0,
            threshold: 1.,
        }
    }
    pub fn add_events_entry(&mut self, step: u32) {
        println!("\tadding events entry for time step {step}");
        self.spikes_queue.push(step);
        self.spikes_queue.sort();
    }
    pub fn get_earliest_event_int(&self) -> Option<&u32> {
        self.spikes_queue.first()
    }
    pub fn pop_earliest_event_int(&mut self) {
        self.spikes_queue.remove(0);
    }
    pub fn plan_init_impulses(&mut self, time_steps: Vec<u32>) {
        self.planned_time_steps = time_steps;
    }
}

impl Clone for LifNeuron {
    fn clone(&self) -> Self {
        Self {
            beta: self.beta,
            threshold: self.threshold,
            current_potential: self.current_potential,
            last_leak_time: self.last_leak_time,
            spikes_queue: self.spikes_queue.clone(),
            id: self.id,
            planned_time_steps: self.planned_time_steps.clone(),
        }
    }
}

impl Init for LifNeuron {
    fn init(&mut self) {
        println!(
            "\t{}\tCalled init!. My leak is {}",
            self.current_potential, self.beta
        );
        for time_step in self.planned_time_steps.clone() {
            self.emmit_signal(time_step);
        }
    }
}

impl PlansEvents for LifNeuron {
    fn get_earliest_event(&self) -> Option<&u32> {
        self.get_earliest_event_int()
    }
    fn pop_earliest_event(&mut self) {
        self.pop_earliest_event_int();
    }
    fn get_earliest_event_available(&self) -> Option<bool> {
        match self.get_earliest_event_int() {
            Some(_) => Some(true),
            None => Some(false),
        }
    }
}

impl Leaky for LifNeuron {
    fn perform_leak(&mut self, time_step: u32) {
        self.current_potential *= self
            .beta
            .powi(time_step.abs_diff(self.last_leak_time).try_into().unwrap());
        self.last_leak_time = time_step;
    }
}

impl HasId for LifNeuron {
    fn set_id(&mut self, id: NeuronUniqueId) {
        self.id = id;
    }

    fn get_id(&self) -> Option<u32> {
        Some(self.id)
    }
}

impl Fire for LifNeuron {
    fn emmit_signal(&mut self, time_step: u32) {
        println!("\t{}\tCalled emmit registrator", self.current_potential);
        self.add_events_entry(time_step);
    }

    fn check_if_should_fire(&mut self, time_step: u32) {
        if self.current_potential >= self.threshold {
            self.emmit_signal(time_step);
            self.current_potential = 0.;
        }
    }
}

impl SignalReceiver for LifNeuron {
    fn get_signal(&self) -> Option<f32> {
        Some(self.current_potential)
    }

    fn recieve_signal(&mut self, time_step: u32, signal: f32) {
        // self.perform_leak(time_step);
        println!(
            "\t{}\trecieved signal of strength: {signal}!",
            self.current_potential
        );
        self.current_potential += signal;
        self.check_if_should_fire(time_step);
    }
}

impl Neuron for LifNeuron {}

impl CommonlyCreateable for LifNeuron {
    fn create_new(beta: f32) -> Self {
        Self::new(beta)
    }
    fn batch_create_new(batch_size: usize, beta: f32) -> Vec<Self> {
        vec![Self::new(beta); batch_size]
    }
}

impl TimeDependent for LifNeuron {
    fn register(self, director: &mut Director) -> Option<NeuronUniqueId> {
        let passed_neuron_trait: Arc<Mutex<dyn Neuron>> = Arc::new(Mutex::new(self));
        director.add_to_registry(passed_neuron_trait) // todo: add meaningfull error handling
    }
    fn register_batch(neurons_batch: Vec<Self>, director: &mut Director) -> Vec<NeuronUniqueId>
    where
        Self: std::marker::Sized,
    {
        neurons_batch
            .into_iter()
            .map(|neuron| neuron.register(director).unwrap())
            .collect()
    }
}
