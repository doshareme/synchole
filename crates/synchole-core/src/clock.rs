use serde::{Deserialize, Serialize};

use crate::DeviceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp {
    pub wall_time_ms: u64,
    pub logical: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridLogicalClock {
    device_id: DeviceId,
    last: Timestamp,
}

impl HybridLogicalClock {
    pub fn new(device_id: DeviceId) -> Self {
        Self {
            device_id,
            last: Timestamp::default(),
        }
    }

    pub fn tick(&mut self, wall_time_ms: u64) -> Timestamp {
        if wall_time_ms > self.last.wall_time_ms {
            self.last = Timestamp {
                wall_time_ms,
                logical: 0,
            };
        } else {
            self.last.logical = self.last.logical.saturating_add(1);
        }
        self.last
    }

    pub fn observe(&mut self, remote: Timestamp, wall_time_ms: u64) -> Timestamp {
        let max_wall = self
            .last
            .wall_time_ms
            .max(remote.wall_time_ms)
            .max(wall_time_ms);
        let logical = if max_wall == self.last.wall_time_ms && max_wall == remote.wall_time_ms {
            self.last.logical.max(remote.logical).saturating_add(1)
        } else if max_wall == self.last.wall_time_ms {
            self.last.logical.saturating_add(1)
        } else if max_wall == remote.wall_time_ms {
            remote.logical.saturating_add(1)
        } else {
            0
        };
        self.last = Timestamp {
            wall_time_ms: max_wall,
            logical,
        };
        self.last
    }

    pub fn device_id(&self) -> &DeviceId {
        &self.device_id
    }

    pub fn last(&self) -> Timestamp {
        self.last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_ticks_logically_when_wall_time_does_not_advance() {
        let mut clock = HybridLogicalClock::new(DeviceId::from("a"));

        assert_eq!(clock.tick(10).logical, 0);
        assert_eq!(clock.tick(10).logical, 1);
        assert_eq!(clock.tick(9).logical, 2);
    }
}
