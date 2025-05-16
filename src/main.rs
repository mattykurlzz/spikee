use crate::neural_sim::TimeDependent;
use neural_sim::time_management::Director;
use neural_sim::LifNeuron;
use neural_sim::time_management::Simulation;

mod neural_sim;

fn main() {
    let sim_time: u32 = 100; // sim time == 100 ticks
    let mut sim: Simulation = Simulation::new(sim_time).expect("Couldn't create sim");

    let director: &mut Director = Director::new(&mut sim).expect("None Value for Director!");

    let neuron: LifNeuron = LifNeuron::new(0.1);
    neuron.register(director);
}
