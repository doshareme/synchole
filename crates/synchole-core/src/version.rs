use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::DeviceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Causality {
    Equal,
    Before,
    After,
    Concurrent,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionVector {
    entries: BTreeMap<DeviceId, u64>,
}

impl VersionVector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self, device_id: DeviceId) -> u64 {
        let counter = self.entries.entry(device_id).or_insert(0);
        *counter = counter.saturating_add(1);
        *counter
    }

    pub fn get(&self, device_id: &DeviceId) -> u64 {
        self.entries.get(device_id).copied().unwrap_or(0)
    }

    pub fn merge(&mut self, other: &Self) {
        for (device, counter) in &other.entries {
            let local = self.entries.entry(device.clone()).or_insert(0);
            *local = (*local).max(*counter);
        }
    }

    pub fn compare(&self, other: &Self) -> Causality {
        let mut less = false;
        let mut greater = false;

        for device in self.entries.keys().chain(other.entries.keys()) {
            let local = self.get(device);
            let remote = other.get(device);
            less |= local < remote;
            greater |= local > remote;
        }

        match (less, greater) {
            (false, false) => Causality::Equal,
            (true, false) => Causality::Before,
            (false, true) => Causality::After,
            (true, true) => Causality::Concurrent,
        }
    }

    pub fn entries(&self) -> &BTreeMap<DeviceId, u64> {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_concurrent_updates() {
        let mut a = VersionVector::new();
        let mut b = VersionVector::new();
        a.increment(DeviceId::from("a"));
        b.increment(DeviceId::from("b"));

        assert_eq!(a.compare(&b), Causality::Concurrent);
    }
}
