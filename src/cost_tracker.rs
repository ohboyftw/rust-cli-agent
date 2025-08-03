use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
pub struct CostTracker {
    total_cost: Arc<Mutex<f64>>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            total_cost: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn add_cost(&self, cost: f64) {
        let mut total_cost = self.total_cost.lock().unwrap();
        *total_cost += cost;
    }

    pub fn get_total_cost(&self) -> f64 {
        *self.total_cost.lock().unwrap()
    }
}