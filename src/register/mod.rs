// Refer to SH2-Reference-Manual

use defmt::Format;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Format)]
#[repr(u8)]
pub enum ReportId {
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
    GameRotationVector = 0x08,      // Report Length 12
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
    ARVRStabilizedRotVec = 0x28,  // Report Length 14
    ARVRStabilizedGameVec = 0x29, // Report Length 12
    GyroIntegratedRotVec = 0x2A,  // Report Length 14
    MotionRequest = 0x2B,         // Report Length 6
    OpticalFlow = 0x2C,           // Report Length 24
    DeadReckoning = 0x2D,         // Report Length 60
    BaseTimestamp = 0xFB,         // Report Length 5
    TimestampRebase = 0xFA,       // Report Length 5
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Status {
    Unreliable = 0,
    LowAccuracy = 1,
    MediumAccuracy = 2,
    HighAccuracy = 3,
    Unknown,
}

impl Format for Status {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Status::Unreliable => defmt::write!(fmt, "Unreliable"),
            Status::LowAccuracy => defmt::write!(fmt, "Low Accuracy"),
            Status::MediumAccuracy => defmt::write!(fmt, "Medium Accuracy"),
            Status::HighAccuracy => defmt::write!(fmt, "High Accuracy"),
            Status::Unknown => defmt::write!(fmt, "Unknown"),
        }
    }
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Register {
    Read(SH2Read),
    Write(SH2Write),
}

// Refer to datasheet 1.3

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum SH2Read {
    ProductIDResponse = 0xF8, // Report Length 16
    FrsReadResponse = 0xF3,   // Report length 16
    CommandResponse = 0xF1,   // Report Length 16
    // Error in SH-2 Reference Manual p. 39 lists GetFeatureResponse as a Write Report ID
    GetFeatureResponse = 0xFC, // Report Length 17
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum SH2Write {
    GetFeatureRequest = 0xFE,
    SetFeatureCommand = 0xFD,
    ProductIDRequest = 0xF9,
    FrsWriteRequest = 0xF7,
    FrsWriteData = 0xF6,
    FrsWriteResponse = 0xF5,
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, defmt::Format)]
pub enum FRSConfiguration {
    AgmStaticCalibration = 0x7979,
    AgmNominalCalibration = 0x4D4D,
    SraStaticCalibration = 0x8A8A,
    SraNominalCalibration = 0x4E4E,
    DynamicCalibration = 0x1F1F,
    MotionEnginePWRMGMT = 0xD3E2,
    SystemOrientation = 0x2D3E,
    PrimaryAccelerometerOrient = 0x2D41,
    GyroscopeOrientation = 0x2D46,
    MagnetometerOrientation = 0x2D4C,
    ARVRStabilizationRotVec = 0x3E2D,
    ARVRStabilizationGameRotVec = 0x3E2E,
    SignificantMotionDetectConf = 0xC274,
    ShakeDetectorConfig = 0x7D7D,
    MaximumFusionPeriod = 0xD7D7,
    SerialNumber = 0x4B4B,
    EnvSensorPressureCalibration = 0x39AF,
    EnvSensorTempCalibration = 0x4D20,
    EnvSensorHumidityCalibration = 0x1AC9,
    EnvSensorAmbLightCalibration = 0x39B1,
    EnvSensorProximityCalibration = 0x4DA2,
    ALSCalibration = 0xD401,
    ProximitySensorCalibration = 0xD402,
    StabilityDetectorConfig = 0xED85,
    UserRecord = 0x74B4,
    MotionEngineTimeSourceSel = 0xD403,
    GyroIntegratedRotVecConfig = 0xA1A2,
}

impl FRSConfiguration {
    pub fn addr(self) -> [u8; 2] {
        let address = self as u16;
        address.to_le_bytes()
    }
}

impl core::convert::TryFrom<u16> for FRSConfiguration {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x7979 => Ok(Self::AgmStaticCalibration),
            0x4D4D => Ok(Self::AgmNominalCalibration),
            0x8A8A => Ok(Self::SraStaticCalibration),
            0x4E4E => Ok(Self::SraNominalCalibration),
            0x1F1F => Ok(Self::DynamicCalibration),
            0xD3E2 => Ok(Self::MotionEnginePWRMGMT),
            0x2D3E => Ok(Self::SystemOrientation),
            0x2D41 => Ok(Self::PrimaryAccelerometerOrient),
            0x2D46 => Ok(Self::GyroscopeOrientation),
            0x2D4C => Ok(Self::MagnetometerOrientation),
            0x3E2D => Ok(Self::ARVRStabilizationRotVec),
            0x3E2E => Ok(Self::ARVRStabilizationGameRotVec),
            0xC274 => Ok(Self::SignificantMotionDetectConf),
            0x7D7D => Ok(Self::ShakeDetectorConfig),
            0xD7D7 => Ok(Self::MaximumFusionPeriod),
            0x4B4B => Ok(Self::SerialNumber),
            0x39AF => Ok(Self::EnvSensorPressureCalibration),
            0x4D20 => Ok(Self::EnvSensorTempCalibration),
            0x1AC9 => Ok(Self::EnvSensorHumidityCalibration),
            0x39B1 => Ok(Self::EnvSensorAmbLightCalibration),
            0x4DA2 => Ok(Self::EnvSensorProximityCalibration),
            0xD401 => Ok(Self::ALSCalibration),
            0xD402 => Ok(Self::ProximitySensorCalibration),
            0xED85 => Ok(Self::StabilityDetectorConfig),
            0x74B4 => Ok(Self::UserRecord),
            0xD403 => Ok(Self::MotionEngineTimeSourceSel),
            0xA1A2 => Ok(Self::GyroIntegratedRotVecConfig),
            _ => Err(()),
        }
    }
}

impl Register {
    pub fn addr(self) -> u8 {
        match self {
            Register::Read(sh2_read) => sh2_read as u8,
            Register::Write(sh2_write) => sh2_write as u8,
        }
    }
}
