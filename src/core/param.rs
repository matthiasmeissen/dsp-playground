use std::sync::{Arc, atomic::{AtomicU32, Ordering}};


/// Shared parameter to change values in the audio thread
#[derive(Clone)]
pub struct SharedParam {
    inner: Arc<AtomicU32>,
}

impl SharedParam {
    pub fn new(initial_value: f32) -> Self {
        Self { inner: Arc::new(AtomicU32::new(initial_value.to_bits())) }
    }

    pub fn set(&self, value: f32) {
        self.inner.store(value.to_bits(), Ordering::Relaxed);
    }

    pub fn get(&self) -> f32 {
        f32::from_bits(self.inner.load(Ordering::Relaxed))
    }
}
