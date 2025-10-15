use crate::config::Coord;
use crate::game::InputDevice;

/// Shared accelerometer data structure for all InputDevice implementations
///
/// This struct can be used by any input device that needs to work with
/// accelerometer data (MPU6050, LSM6DS3, etc.)
///
/// Values are in raw accelerometer units, typically:
/// - Range: -32768 to 32767 for ±2g scale
/// - 1g ≈ 16384 units
pub struct AccelData {
    pub accel_x: i32, // X-axis acceleration
    pub accel_y: i32, // Y-axis acceleration
    pub accel_z: i32, // Z-axis acceleration
}

/// Helper function to convert accelerometer data to game coordinates
///
/// This can be used by any InputDevice implementation that wants to map
/// accelerometer tilt to screen position.
///
/// # Parameters
/// - `accel_data`: Raw accelerometer reading
/// - `y_min`, `y_max`: Game coordinate bounds
/// - `tilt_threshold`: Minimum acceleration to register as "tapped" (typically 8000-12000)
///
/// # Returns
/// - `(mapped_y, is_tilted)`: Y coordinate and whether significant tilt was detected
pub fn accel_to_game_coords(
    accel_data: &AccelData,
    y_min: Coord,
    y_max: Coord,
    tilt_threshold: i32,
) -> (Coord, bool) {
    let is_tilted = accel_data.accel_y.abs() > tilt_threshold;

    // Map accelerometer Y value to screen Y coordinate
    // Scale from accelerometer range (-32768 to 32767) to screen range (y_min to y_max)
    let normalized_y = if accel_data.accel_y > 0 {
        // Positive tilt maps to upper part of range
        let scaled = (accel_data.accel_y * (y_max - y_min)) / 32767;
        y_min + scaled.min(y_max - y_min)
    } else {
        // Negative tilt maps to lower part of range
        let scaled = ((-accel_data.accel_y) * (y_max - y_min)) / 32767;
        y_max - scaled.min(y_max - y_min)
    };

    // Clamp to valid range
    let mapped_y = normalized_y.clamp(y_min, y_max);

    (mapped_y, is_tilted)
}
// Dummy input device for testing
/*pub struct DummyInputDevice;

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
*/
// Real input device using MPU6050
pub struct Mpu6050InputDevice;

impl Mpu6050InputDevice {
    pub fn new() -> Self {
        Self
    }
}

impl InputDevice for Mpu6050InputDevice {
    type Error = ();

    fn init(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_tap(&mut self, y_min: Coord, y_max: Coord) -> Result<(Coord, bool), Self::Error> {
        match crate::mpu6050::read_accel_data() {
            Ok(accel_data) => {
                // Use the shared helper function with appropriate threshold
                let tilt_threshold = 8000; // Adjust based on sensitivity needed
                let (mapped_y, is_tilted) =
                    accel_to_game_coords(&accel_data, y_min, y_max, tilt_threshold);
                Ok((mapped_y, is_tilted))
            }
            Err(_) => {
                // If MPU6050 read fails, return no tap and center position
                Ok(((y_min + y_max) / 2, false))
            }
        }
    }
}

// Example: How other accelerometer-based input devices could use AccelData
/*
pub struct LSM6DS3InputDevice;

impl InputDevice for LSM6DS3InputDevice {
    type Error = ();

    fn init(&mut self) -> Result<(), Self::Error> {
        // Initialize LSM6DS3 sensor
        Ok(())
    }

    fn is_tap(&mut self, y_min: Coord, y_max: Coord) -> Result<(Coord, bool), Self::Error> {
        // Read from LSM6DS3 sensor and create AccelData
        let accel_data = AccelData {
            accel_x: 0, // read from sensor
            accel_y: 0, // read from sensor
            accel_z: 0, // read from sensor
        };

        // Use the shared helper function
        let tilt_threshold = 10000; // Different sensitivity
        let (mapped_y, is_tilted) = accel_to_game_coords(&accel_data, y_min, y_max, tilt_threshold);
        Ok((mapped_y, is_tilted))
    }
}
*/
