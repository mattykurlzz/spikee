use time_management::ControllingUnit;

pub mod time_management;

trait Leaky {
    fn perform_step_leak(self) -> ();
}

pub trait TimeDependent {
    fn register(self, director: &mut time_management::Director);
}

#[derive(Debug)]
pub struct LifNeuron {
    current_potential: f32,
    leak_rate: f32,
}

impl LifNeuron {
    pub fn new(leak: f32) -> Self {
        Self {
            current_potential : 0., 
            leak_rate : leak,
        }
    }
}

impl Leaky for LifNeuron {
    fn perform_step_leak(mut self) -> () {
        self.current_potential -= self.leak_rate
    }
}

impl TimeDependent for LifNeuron {
    fn register(self, director: &mut time_management::Director){
        director.add_to_registry(self);
    }
}
