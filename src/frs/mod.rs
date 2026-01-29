use crate::{
    READ, SensorError, WRITE,
    data::Packet,
    register::{FRSConfiguration, Register, SH2Read, SH2Write},
};

#[derive(Copy, Clone, Debug, defmt::Format)]
pub enum FRSStatus {
    NoError,
    UnrecognizedFRSType,
    Busy,
    ReadRecordCompleted,
    Deprecated,
    RecordEmpty,
    DeviceError,
    InvalidResponse,
    Reserved,
}

fn process_status(status: u8) -> FRSStatus {
    let status_num = status & 0b00001111;
    match status_num {
        0 => FRSStatus::NoError,
        1 => FRSStatus::UnrecognizedFRSType,
        2 => FRSStatus::Busy,
        3 => FRSStatus::ReadRecordCompleted,
        4 => FRSStatus::Deprecated,
        5 => FRSStatus::RecordEmpty,
        6 => FRSStatus::Deprecated,
        7 => FRSStatus::Deprecated,
        8 => FRSStatus::DeviceError,
        9..16 => FRSStatus::Reserved,
        _ => FRSStatus::InvalidResponse,
    }
}

#[derive(Debug, defmt::Format)]
pub struct FRSDataRead {
    request_type: FRSConfiguration,
    length: Option<u8>,
    offset: Option<u16>,
    status: Option<FRSStatus>,
    data_0: Option<u32>,
    data_1: Option<u32>,
}

impl FRSDataRead {
    pub fn new(request: FRSConfiguration) -> Self {
        FRSDataRead {
            request_type: request,
            length: None,
            offset: None,
            status: None,
            data_0: None,
            data_1: None,
        }
    }

    pub fn generate_read_request(&self) -> Result<[u8; 8], crate::SensorError> {
        let addr_bytes: &[u8; 2] = &self.request_type.addr();
        Ok([
            Register::Write(SH2Write::FrsReadRequest).addr(),
            0,
            0,
            0,
            addr_bytes[0],
            addr_bytes[1],
            0,
            0,
        ])
    }

    pub fn process_read_response(&mut self, data: &[u8]) -> Result<(), SensorError> {
        if data.len() >= 16 && data[0] == Register::Read(SH2Read::FrsReadResponse).addr() {
            self.length = Some(data[1] >> 4);
            self.status = Some(process_status(data[1]));
            self.offset = Some(u16::from_le_bytes(
                data[2..4].try_into().expect("Failed to slice"),
            ));
            if let Some(length) = self.length
                && length > 0
            {
                self.data_0 = Some(u32::from_le_bytes(
                    data[4..8].try_into().expect("Failed to slice."),
                ));
            }
            if let Some(length) = self.length
                && length > 1
            {
                self.data_1 = Some(u32::from_le_bytes(
                    data[8..12].try_into().expect("Failed to slice."),
                ));
            }
            let request = u16::from_le_bytes(data[12..14].try_into().expect("Failed to slice."));
            let request =
                FRSConfiguration::try_from(request).expect("Failed to find configuration");

            if request == self.request_type {
                Ok(())
            } else {
                Err(SensorError::Unimplemented)
            }
        } else {
            Err(SensorError::InvalidLength)
        }
    }
}

#[derive(Debug, defmt::Format)]
pub struct FRSDataWrite {
    request_type: FRSConfiguration,
    data_ready: bool,
    length: u8,
    offset: Option<u16>,
    data_0: Option<u32>,
    data_1: Option<u32>,
}

impl FRSDataWrite {
    pub fn new(request: FRSConfiguration, data_0: Option<u32>, data_1: Option<u32>) -> Self {
        let mut data_length: u8 = 0;
        if data_0.is_some() {
            data_length += 1;
        }
        if data_1.is_some() {
            data_length = 2;
        }

        FRSDataWrite {
            request_type: request,
            data_ready: true,
            length: data_length,
            offset: None,
            data_0,
            data_1,
        }
    }
    // pub fn generate_write_request(&self) -> Result<[u8; 6], crate::SensorError> {
    //     if self.read_write == WRITE {
    //         let addr_bytes: &[u8; 2] = &self.request_type.addr();
    //         Ok([
    //             Register::Write(SH2Write::FrsReadRequest).addr(),
    //             0,

    //             addr_bytes[0],
    //             addr_bytes[1],
    //         ])
    //     } else {
    //         Err(crate::SensorError::ReadWriteFlipped)
    //     }
    // }
}
