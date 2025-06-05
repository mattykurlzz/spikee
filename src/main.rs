use neural_sim::neuron::TimeDependent;
use neural_sim::{ControllingUnit, Director};
use neural_sim::neuron::LifNeuron;
use neural_sim::Simulation;

mod neural_sim;

fn main() {
    let sim_time: u32 = 15; // sim time == 100 ticks
    let mut sim: Simulation = Simulation::new().expect("Couldn't create sim");

    let director: Director = Director::new(sim_time).unwrap();
    let director: &mut Director = sim.register_director(director).unwrap();

    let mut neuron_1: LifNeuron = LifNeuron::new(0.9);
    let neuron_2: LifNeuron = LifNeuron::new(0.85);
    neuron_1.plan_init_impulses(vec![0, 2, 5, 7]);

    let neuron_1 = neuron_1.register(director).unwrap();
    let neuron_2 = neuron_2.register(director).unwrap();
    
    director.create_link(neuron_1, neuron_2, 0.6);
    
    sim.start();
}
