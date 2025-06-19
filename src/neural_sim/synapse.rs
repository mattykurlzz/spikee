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
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            recepient: self.recepient,
            weight: self.weight,
        }
    }
}

impl Clone for SynapseGroup {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
        }
    }
}
