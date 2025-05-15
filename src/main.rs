use neural_sim::time_management::Director;
use neural_sim::LifNeuron;
use neural_sim::time_management::Simulation;

mod neural_sim;

fn main() {
    let sim_time: u32 = 100; // sim time == 100 ticks
    let mut sim = match Simulation::new(sim_time) {
        Ok(it) => it,
        Err(err) => return eprintln!("{}", err),
    };

    let director = Director::new(&mut sim);
    let neuron = LifNeuron::new(0.1);
    //let director = neuron.register(director);
}
