use defmt::info;
use heapless::{Vec, vec};

pub struct Packet {
    length: u16,
    channel: u8,
    seq_num: u8,
    header: heapless::Vec<u8, 4>,
    data: heapless::Vec<u8, 32763>,
}

impl Packet {
    pub fn new() -> Self {
        Packet {
            length: 0,
            channel: 6,
            seq_num: 0,
            header: Vec::from_array([0; 4]),
            data: Vec::new(),
        }
    }

    pub fn from_buf(buf: &[u8]) -> Self {
        let mut temp = Packet {
            length: 0,
            channel: 6,
            seq_num: 0,
            header: Vec::from_slice(&buf[..4]).expect("Packet creation error"),
            data: Vec::from_slice(&buf[4..]).expect("Packet creation error"),
        };
        temp.process_header(false);
        temp
    }

    pub fn from_data_buf(
        buf: &[u8],
        channel: u8,
        seq_num: u8,
    ) -> Result<Self, crate::data::PacketError> {
        let mut temp = Packet {
            length: 0,
            channel: channel,
            seq_num: 0,
            header: Vec::from_array([0; 4]),
            data: Vec::from_slice(buf).expect("Packet creation error"),
        };
        temp.generate_header(buf);
        if channel < 6 {
            Ok(temp)
        } else {
            Err(PacketError::InvalidChannel)
        }
    }

    fn generate_header(&mut self, data_buf: &[u8]) {
        let length = data_buf.len() as u16 + 4;
        let length_slice: [u8; 2] = length.to_le_bytes();
        self.header[0] = length_slice[0];
        self.header[1] = length_slice[1];
        self.header[2] = self.channel;
        self.seq_num = 0;
        self.header[3] = self.seq_num;
    }

    pub fn as_mut_header(&mut self) -> &mut [u8] {
        &mut self.header
    }

    pub fn process_header(&mut self, resize: bool) {
        info!("HEADER: {:#X}", self.header.as_slice());
        self.calculate_length();
        // info!("LENGTH DATA: {}", self.length);
        if resize {
            self.data.resize(self.data_length() as usize, 0);
        }
        self.channel = u8::from_le_bytes([self.header[2]]);
        self.seq_num = u8::from_le_bytes([self.header[3]]);
    }

    fn calculate_length(&mut self) -> Result<(), PacketError> {
        let length_slice: [u8; 2] = self.header[..2].try_into().expect("Failed to slice header");
        let value = u16::from_le_bytes(length_slice);

        self.length = value & 0x7FFF;

        // Packet is a continuation
        if (value >> 15) & 1 == 1 {
            Err(PacketError::HalfPacket)
        } else {
            Ok(())
        }
    }

    pub fn packet_length(&self) -> u16 {
        self.length
    }

    pub fn data_length(&self) -> u16 {
        if self.length > 4 { self.length - 4 } else { 0 }
    }

    pub fn get_data_report(&mut self) -> (u16, &mut [u8]) {
        (self.data_length(), &mut self.data)
    }

    pub fn as_mut_data(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn full_packet(&mut self) -> Vec<u8, 32767> {
        let mut temp: Vec<u8, 32767> =
            Vec::from_slice(self.as_mut_header()).expect("Failed to create full packet");
        temp.extend_from_slice(self.as_mut_data())
            .expect("Failed to attach data slice to full packet");
        temp
    }

    pub fn channel(&self) -> u8 {
        self.channel
    }

    pub fn seq_num(&self) -> u8 {
        self.seq_num
    }

    pub fn report_id(&self) -> u8 {
        if self.data_length() > 0 {
            0
            // self.data.get(0).unwrap_or(&0).clone()
        } else {
            0
        }
    }
}

#[derive(Debug)]
pub enum PacketError {
    HalfPacket,
    InvalidChannel,
}

pub struct VarBuf {
    buf: [u8; 256],
    valid_data: usize,
}

impl VarBuf {
    pub fn new() -> Self {
        VarBuf {
            buf: [0; 256],
            valid_data: 255,
        }
    }

    pub fn push(&mut self, byte: u8) {
        if (self.valid_data) < self.buf.len() {
            self.buf[self.valid_data] = byte;
        }
    }

    pub fn as_slice(&mut self) -> &[u8] {
        &self.buf[..self.valid_data]
    }

    pub fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.valid_data]
    }

    pub fn into_inner(self) -> [u8; 256] {
        self.buf
    }

    pub fn clone_buf(self) -> [u8; 256] {
        self.buf.clone()
    }
}
