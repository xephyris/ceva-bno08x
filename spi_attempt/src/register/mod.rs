// Refer to datasheet 1.3

#[allow(dead_code)]
#[repr(u8)]
pub enum Register {
    Read(SH2Read),
    Write(SH2Write),
    Report(ReportID),
}

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

pub enum ReportID {
    RotationVector = 0x05,
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
            Register::Report(report_id) => report_id as u8,
        }
    }
}
