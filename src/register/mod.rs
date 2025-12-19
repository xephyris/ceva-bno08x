// Refer to SH2-Reference-Manual

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ReportID {
    AccelerometerRaw = 0x14,        // Report Length 16
    AccelerometerCalibrated = 0x01, // Report Length 10
    AccelerationLinear = 0x04,      // Report Length 10
    Gravity = 0x06,                 // Report Length 10
    GyroscopeRaw = 0x15,            // Report Length 16
    GyroscopeCalibrated = 0x02,     // Report Length 10
    GyroscopeUncalibrated = 0x07,   // Report Length 16
    MagnetometerRaw = 0x16,         // Report Length 16
    MagFieldCalibrated = 0x03,      // Report Length 10
    MagFieldUncalibrated = 0x0F,    // Report Length 16
    RotationVector = 0x05,          // Report Length 14
    GeomagneticRotVector = 0x09,    // Report Length 14
    Pressure = 0x0A,                // Report Length 8
    AmbientLight = 0x0B,            // Report Length 8
    Humidity = 0x0C,                // Report Length 6
    Proximity = 0x0D,               // Report Length 6
    Tempterature = 0x0E,            // Report Length 6
    Reserved = 0x17,
    TapDetector = 0x10,           // Report Length 5
    StepDetector = 0x18,          // Report Length 8
    StepCounter = 0x11,           // Report Length 12
    SignificantMotion = 0x12,     // Report Length 6
    StabilityClassifier = 0x13,   // Report Length 6
    ShakeDetector = 0x19,         // Report Length 6
    FlipDetector = 0x1A,          // Report Length 6
    PickupDetector = 0x1B,        // Report Length 6
    StabilityDetector = 0x1C,     // Report Length 5
    PersonalActClassifier = 0x1E, // Report Length 16
    SleepDetector = 0x1F,         // Report Length 6
    TiltDetector = 0x20,          // Report Length 6
    PocketDetector = 0x21,        // Report Length 6
    CircleDetector = 0x22,        // Report Length 6
    HeartRateMonitor = 0x23,      // Report Length 6
    ARVRStabilizedVec = 0x28,     // Report Length 14
    ARVRStabilizedGameVec = 0x29, // Report Length 12
    GyroIntegratedRotVec = 0x2A,  // Report Length 14
    MotionRequest = 0x2B,         // Report Length 6
    OpticalFlow = 0x2C,           // Report Length 24
    DeadReckoning = 0x2D,         // Report Length 60
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Register {
    Read(SH2Read),
    Write(SH2Write),
}

// Refer to datasheet 1.3

#[allow(dead_code)]
#[repr(u8)]
pub enum SH2Read {
    GetFeatureResponse = 0xFC,
    ProductIDResponse = 0xF8,
    FrsWriteResponse = 0xF5,
    FrsReadResponse = 0xF3,
    CommandResponse = 0xF1,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum SH2Write {
    GetFeatureRequest = 0xFE,
    SetFeatureCommand = 0xFD,
    ProductIDRequest = 0xF9,
    FrsWriteRequest = 0xF7,
    FrsWriteData = 0xF6,
    FrsReadRequest = 0xF4,
    CommandRequest = 0xF2,
}

#[repr(u8)]
pub enum I2CAddress {
    Default = 0x4A,
    Alternate = 0x4B,
}

#[allow(dead_code)]
#[repr(u16)]
pub enum Configuration {
    AgmStaticCalibration = 0x7979,
    AgmNominalCalibration = 0x4D4D,
    SraStaticCalibration = 0x8A8A,
    SraNominalCalibration = 0x4E4E,
}

impl Register {
    pub fn addr(self) -> u8 {
        match self {
            Register::Read(sh2_read) => sh2_read as u8,
            Register::Write(sh2_write) => sh2_write as u8,
        }
    }
}
