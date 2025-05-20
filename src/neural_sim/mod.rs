use time_management::{ControllingUnit, event_schedule_entry::EventEntry};

pub mod time_management;

pub trait TimeDependent<F> 
where F: FnOnce(i32){
    fn register(self, director: &mut time_management::Director<F>, init: bool);
}

pub trait Neuron<'a> {
    fn init(&self, time_step: i32);
    fn recieve_signal(&self, time_step: i32, signal: f32);
    fn emmit_signal(&self, time_step: i32);
}

#[derive(Debug)]
pub struct LifNeuron {
    current_potential: f32,
    leak_rate: f32,
}

impl LifNeuron {
    pub fn new(leak: f32) -> Self {
        Self {
            current_potential: 0.,
            leak_rate: leak,
        }
    }
}

impl<'a> Neuron<'a> for LifNeuron {
    fn init(&self, time_step: i32) {
        println!("Called init!. My leak is {}", self.leak_rate);
    }
    fn recieve_signal(&self, time_step: i32, signal: f32) {
        println!("Called recv_sig");
    }
    fn emmit_signal(&self, time_step: i32) {
        println!("Called emmit");
    }
}
// impl Leaky for LifNeuron {
//     fn perform_step_leak(mut self) -> () {
//         self.current_potential -= self.leak_rate
//     }
// }

impl<F> TimeDependent<F> for LifNeuron 
where F: FnOnce(i32) {
    fn register(self, director: &mut time_management::Director<F>, init: bool) {
        let lif_ref = director.add_to_registry(self).expect("Err"); // todo: add meaningfull error handling
        if init {
            let closure = |time| lif_ref.init(time);
            let event: EventEntry<F> =
                EventEntry::<F>::new(0, closure).expect("Error creating event");
            // director.add_event();
        }
    }
}
