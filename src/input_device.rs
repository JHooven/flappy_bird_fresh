use crate::config::Coord;
use crate::game::InputDevice;

// Dummy input device for testing
pub struct DummyInputDevice;

impl DummyInputDevice {
    pub fn new() -> Self {
        DummyInputDevice
    }
}

impl InputDevice for DummyInputDevice {
    type Error = ();

    fn init(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_tap(&mut self, _y_min: Coord, _y_max: Coord) -> Result<(Coord, bool), Self::Error> {
        // Always return "no tap" and dummy X-coordinate
        Ok((100, false))
    }
}
