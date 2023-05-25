use std::sync::Arc;

use crate::connection::OutBound;

pub struct Group();

impl Group {
    pub fn new(name: Arc<String>) -> Self {
        Self()
    }

    pub fn join(&self, outbound: Arc<OutBound>) {}

    pub fn post(&self, message: Arc<String>) {}
}
