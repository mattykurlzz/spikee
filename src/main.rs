use neural_sim::{ControllingUnit, Director, VecOrValueFloat, BatchLinkingRule};
use neural_sim::neuron::lif_neuron::LifNeuron;
use neural_sim::error::Error;
use neural_sim::Simulation;

use crate::neural_sim::neuron::{CommonlyCreateable, TimeDependent};

mod neural_sim;

fn main() -> Result<(), Error> {
    let sim_time: u32 = 15;
    let lin_layers_size: [u32; 2] = [10, 10];
    let mut sim: Simulation = Simulation::new(true, None).unwrap();

    let director: Director = Director::new(sim_time, 0).unwrap();
    let director: &mut Director = sim.register_director(director).unwrap();

    let mut layer_1 = LifNeuron::batch_create_new(lin_layers_size[0].try_into().unwrap(), 0.6);
    let layer_2 = LifNeuron::batch_create_new(lin_layers_size[1].try_into().unwrap(), 0.6);

    for neuron in &mut layer_1 {
        neuron.plan_init_impulses(vec![1, 3]);
    }

    let layer_1 = LifNeuron::register_batch(layer_1, director);
    let layer_2 = LifNeuron::register_batch(layer_2, director);

    director.create_links_by_rule(
        &layer_1,
        &layer_2,
        VecOrValueFloat::Val(0.3),
        BatchLinkingRule::UserDefined(|x, y| x == y),
    )?;
    director.create_links_by_rule(
        &layer_1[0..1],
        &layer_2,
        VecOrValueFloat::Val(0.5),
        BatchLinkingRule::FullyConnected,
    )?;

    sim.start()?;

    Ok(())
}
