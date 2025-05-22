use neural_sim::TimeDependent;
use neural_sim::time_management::Director;
use neural_sim::LifNeuron;
use neural_sim::time_management::Simulation;

mod neural_sim;

fn main() {
    let sim_time: u32 = 10; // sim time == 100 ticks
    let mut sim: Simulation = Simulation::new().expect("Couldn't create sim");

    let director: Director = Director::new(sim_time).expect("None Value for Director!");
    let director: &mut Director = sim.register_director(director).expect("{ErrorKind::Other}" );

    let neuron_1: LifNeuron = LifNeuron::new(0.1);
    let neuron_2: LifNeuron = LifNeuron::new(0.15);
    neuron_1.register(director);
    neuron_2.register(director);
    
    sim.start();
}
