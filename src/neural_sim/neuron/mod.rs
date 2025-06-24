use super::{Director, NeuronUniqueId};

pub mod lif_neuron;

pub trait TimeDependent {
    fn register(self, director: &mut Director) -> Option<NeuronUniqueId>;
    fn register_batch(neurons_batch: Vec<Self>, director: &mut Director) -> Vec<NeuronUniqueId>
    where
        Self: std::marker::Sized;
}

#[allow(dead_code)] // create_new not used, but should be tested //todo
pub trait CommonlyCreateable {
    fn create_new(beta: f32) -> Self;
    fn batch_create_new(batch_size: usize, beta: f32) -> Vec<Self>
    where
        Self: std::marker::Sized;
}

pub trait SignalReceiver{
    fn recieve_signal(&mut self, time_step: u32, signal: f32); //neuron
    fn get_signal(&self) -> Option<f32>; // Neuron
}

pub trait Init{
    fn init(&mut self);
}

pub trait HasId {
    fn set_id(&mut self, id: NeuronUniqueId); 
    fn get_id(&self) -> Option<u32>; 
}

pub trait Fire: HasId {
    fn emmit_signal(&mut self, time_step: u32);
    fn fire(&self) -> Option<NeuronUniqueId> {
        self.get_id()
    }
    fn check_if_should_fire(&mut self, time_step: u32);
}

pub trait Leaky {
    fn perform_leak(&mut self, time_step: u32); // Neuron
}

pub trait PlansEvents {
    fn get_earliest_event(&self) -> Option<&u32>; 
    fn get_earliest_event_available(&self) -> Option<bool>; 
    fn pop_earliest_event(&mut self); 
}

pub trait Neuron: Send + Sync + SignalReceiver + Init + HasId + Fire + Leaky + PlansEvents {}