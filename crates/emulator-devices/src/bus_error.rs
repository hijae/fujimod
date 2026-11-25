//! Bus-error region model for architectural external aborts on access.

use crate::checkpoint::Checkpoint;
use crate::{Cx, Device};
use serde_json::{Value, json};

/// Stateless memory-mapped region that raises an architectural bus error on
/// every guest read or write without inspecting processor state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BusError;

impl Device for BusError {
    fn name(&self) -> &'static str {
        "bus-error"
    }

    fn read(&mut self, cx: &mut Cx<'_>, _address: u64, _size: u8) -> u64 {
        cx.bus_error();
        0
    }

    fn write(&mut self, cx: &mut Cx<'_>, _address: u64, _size: u8, _value: u64) {
        cx.bus_error();
    }

    fn save(&self) -> Value {
        json!({})
    }

    fn checkpoint(&self) -> Result<Checkpoint, String> {
        Checkpoint::encode(self.name(), &())
    }

    fn validate_checkpoint(&self, saved: &Checkpoint) -> Result<(), String> {
        saved.decode::<()>(self.name())
    }

    fn restore_checkpoint(&mut self, saved: &Checkpoint) -> Result<(), String> {
        self.validate_checkpoint(saved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;

    #[test]
    fn bus_error_device_marks_reads_and_writes_without_refusal_or_unmodeled() {
        let mut host = TestHost::default();
        let mut device = BusError;

        let mut read_cx = Cx::new(&mut host, 0, 0x1000, 0);
        assert_eq!(device.read(&mut read_cx, 0xfff1_0000, 4), 0);
        assert!(read_cx.is_bus_error());
        assert!(!read_cx.is_unmodeled());
        assert!(read_cx.refusal().is_none());

        let mut write_cx = Cx::new(&mut host, 0, 0x1004, 0);
        device.write(&mut write_cx, 0xfff1_0000, 4, 0x1234_5678);
        assert!(write_cx.is_bus_error());
        assert!(!write_cx.is_unmodeled());
        assert!(write_cx.refusal().is_none());
    }

    #[test]
    fn bus_error_device_supports_checkpoint_round_trip() {
        let mut device = BusError;
        let saved = device.checkpoint().unwrap();
        device.validate_checkpoint(&saved).unwrap();
        device.restore_checkpoint(&saved).unwrap();
        assert_eq!(device.save(), json!({}));
    }
}
