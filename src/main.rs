use neural_sim::neuron::TimeDependent;
use neural_sim::{ControllingUnit, Director, VecOrValueFloat, BatchLinkingRule};
use neural_sim::neuron::LifNeuron;
use neural_sim::Simulation;

use crate::neural_sim::neuron::CommonlyCreateable;
mod neural_sim;

fn main() -> std::io::Result<()>{
    // {
    //     let sim_time: u32 = 15; // sim time == 100 ticks
    //     let mut sim: Simulation = Simulation::new().expect("Couldn't create sim");

    //     let director: Director = Director::new(sim_time).unwrap();
    //     let director: &mut Director = sim.register_director(director).unwrap();

    //     let mut neuron_1: LifNeuron = LifNeuron::new(0.9);
    //     let neuron_2: LifNeuron = LifNeuron::new(0.85);
    //     neuron_1.plan_init_impulses(vec![0, 2, 5, 7]);

    //     let neuron_1 = neuron_1.register(director).unwrap();
    //     let neuron_2 = neuron_2.register(director).unwrap();
        
    //     director.create_link(neuron_1, neuron_2, 0.6);
        
    //     sim.start();
    // }
    

    {
        let sim_time: u32 = 15; 
        let lin_layers_size: [u32; 2] = [2, 2];
        let mut sim: Simulation = Simulation::new(true, None).unwrap();

        let director: Director = Director::new(sim_time, 0).unwrap(); 
        let director: &mut Director = sim.register_director(director).unwrap();
        
        let layer_1 = LifNeuron::batch_create_new(lin_layers_size[0].try_into().unwrap(), 0.6);
        let layer_2 = LifNeuron::batch_create_new(lin_layers_size[1].try_into().unwrap(), 0.6);
        
        let layer_1 = LifNeuron::register_batch(layer_1, director);
        let layer_2 = LifNeuron::register_batch(layer_2, director);
        
        director.create_links_by_rule(layer_1, 
            layer_2, 
            VecOrValueFloat::Val(0.6), 
            BatchLinkingRule::FullyConnected
        );
        
        sim.start();
        
        Ok(())
    }
}
