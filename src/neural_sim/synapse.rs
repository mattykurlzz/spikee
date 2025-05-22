#[derive(Debug)]
pub struct Connection {
  recepient: u32, 
  weight: f32,
}

#[derive(Debug)]
pub struct SynapseGroup {
  connections: Vec<Connection>,
}

impl SynapseGroup {
  pub fn new() -> Option<Self> {
    Some(Self {
      connections: Vec::new(),
    })
  }
  pub fn fire(&self) -> &Vec<Connection> {
    &self.connections
  }
}

