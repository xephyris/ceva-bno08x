#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;
use embedded_hal::{self as hal};
use panic_probe as _;

use hal::digital::InputPin;
use hal::spi::*;

use defmt::*;

use crate::data::{Packet, ProductId, VarBuf};
use crate::register::*;

pub mod data;
pub mod error;
mod register;

const WRITE: bool = true;
const READ: bool = false;
const TIMEOUT: u32 = 2000000;

// BAUD RATE MUST BE 100000 HZ AT 3MHZ SPI FREQUENCY!!!!!!
#[derive(Debug)]
pub struct BNO08x<I2C, D> {
    address: u8,
    i2c: I2C,
    delay: D,
    seq_num_w: [u8; 6],
    seq_num_r: [u8; 6],
}

impl<I2C, D> BNO08x<I2C, D>
where
    I2C: I2c,
    D: DelayNs,
{
    pub fn new(i2c: I2C, delay: D, default_addr: bool) -> Self {
        BNO08x {
            address: if default_addr {
                I2CAddress::Default as u8
            } else {
                I2CAddress::Alternate as u8
            },
            i2c,
            delay,
            seq_num_w: [0; 6],
            seq_num_r: [0; 6],
        }
    }

    fn read_header(&mut self) -> Packet {
        let mut header = [0u8; 4];
        self.i2c.read(self.address, &mut header).ok();
        println!(
            "RAW PACKET ************************ RAW PACKET \n HEADER: {}",
            header
        );

        Packet::from_header(&header, true)
    }

    pub fn soft_reset_device(&mut self) {
        let mut write =
            Packet::from_data_buf(&[0x01], 1, self.seq_num_w[1], false).expect("PACK GEN ERROR");
        self.increment_seq_num(WRITE, 1, None);

        self.i2c
            .write(self.address, write.full_packet().as_slice())
            .ok();

        self.delay.delay_ms(500);
        self.i2c
            .write(self.address, write.full_packet().as_slice())
            .ok();
        self.delay.delay_ms(500);

        for _ in 0..3 {
            let mut packet = self.read_packet();
            // info!("AFTER REST: {}",  packet.full_packet().as_slice())
        }
    }

    pub fn read_packet(&mut self) -> Packet {
        let mut out = self.read_header();
        out.process_header(true);
        info!("R PACK LENGTH: {}", out.packet_length());
        info!("R CHANNEL NUM: {}", 0);
        info!("R SEQ NUM: {}", out.seq_num());
        info!(
            "R CHANNEL {} HAS {} BYTES AVAILABLE",
            out.channel(),
            out.data_length()
        );
        self.seq_num_r[0 as usize] = out.seq_num();

        if out.packet_length() == 0 {
            info!("NO PACKET AVAILABLE");
        }
        self.i2c.read(self.address, out.as_mut_data(true)).ok();

        if out.channel() != 3 {
            info!("FULL PACKET DATA {:#X}", out.as_mut_data(true).len());
        }
        out
    }

    pub fn send_packet(&mut self, channel: u8, data: &[u8]) {
        let seq = self.increment_seq_num(WRITE, channel, None);
        let mut write = Packet::from_data_buf(data, channel, seq, false).expect("PacketGen failed");
        info!(
            "SENDING PACKETS WITH CONTENTS {}",
            write.full_packet().as_slice()
        );
        let mut read = Packet::from_header(write.as_mut_header(), false);
        println!("READ: {}", read.full_packet().as_slice());
        self.i2c
            .write(self.address, write.full_packet().as_slice())
            .ok();
        info!("PACKET SENT");
    }

    pub fn read_product_id(&mut self) -> Result<bool, MyError> {
        // let mut buf = Packet::from_data_buf(
        //     &[Register::Write(SH2Write::ProductIDRequest).addr(), 0x00],
        //     2,
        //     self.seq_num_w[2],
        // )
        // .expect("W Packet Channel Invalid");
        info!("READING P ID");
        let mut buf_data = [Register::Write(SH2Write::ProductIDRequest).addr(), 0x00];
        self.send_packet(2, &buf_data);
        // self.increment_seq_num(WRITE, 2, None);

        let mut retries = 0;
        // let max = 3;
        let mut out = Packet::new(true);
        while out.channel() != 2
            && out.report_id() != Register::Write(SH2Write::ProductIDRequest).addr()
        {
            retries += 1;
            out = self.read_packet();
            info!(
                "OUT CHANNEL IS {} \n OUT REPORT ID IS {:#X}",
                out.channel(),
                out.report_id()
            );
        }

        let product_id = ProductId::new(out.as_mut_data(false));
        info!("{:?}", product_id.display());

        if product_id.display().0 != (0, 0) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn enable_quaternions(&mut self) -> bool {
        info!("ENABLING QUATERNIONS");
        // let mut buf_data = [
        //     Register::Write(SH2Write::SetFeatureCommand).addr(),
        //     Register::Report(ReportID::RotationVector).addr(),
        // ];

        // TODO Remove hardcoding and replace with constants
        let mut buf_data = [
            0xFD, 0x05, 0x00, 0x00, 0x00, 0x10, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        self.send_packet(2, &buf_data);
        let mut retries = 0;
        let mut out = Packet::new(true);
        while out.channel() != 2
            && out.report_id() != Register::Read(SH2Read::GetFeatureResponse).addr()
        {
            retries += 1;
            out = self.read_packet();
            info!(
                "OUT CHANNEL IS {} \n OUT REPORT ID IS {:#X}",
                out.channel(),
                out.report_id()
            );
        }
        // info!("FEATURE REQUEST RESPONSE: {:#X}", out.as_mut_data());
        // info!("R PID {:#X}", out.full_packet().as_slice());
        true
    }

    pub fn quaternions(&mut self) -> (u32, u32, u32, u32) {
        info!("READING QUATERNIONS");
        let mut buf_data = [
            Register::Write(SH2Write::GetFeatureRequest).addr(),
            Register::Report(ReportID::RotationVector).addr(),
        ];
        // self.send_packet(3, &buf_data);
        let mut retries = 0;
        let mut out = Packet::new(true);
        while out.channel() != 3
            && out.report_id() != Register::Read(SH2Read::GetFeatureResponse).addr()
        {
            retries += 1;
            out = self.read_packet();
            info!(
                "OUT CHANNEL IS {} \n OUT REPORT ID IS {:#X}",
                out.channel(),
                out.report_id()
            );
        }
        info!("FEATURE REQUEST RESPONSE: {:#X}", out.as_mut_data(true));
        self.delay.delay_ms(5);
        self.parse_quaternions(out);
        // info!("R PID {:#X}", out.full_packet().as_slice());
        (0, 0, 0, 0)
    }

    fn parse_quaternions(&mut self, mut packet: Packet) -> (f32, f32, f32, f32) {
        // if packet.data_length() == 23 {
        let slice = packet.as_mut_data(false);
        let i_slice: [u8; 2] = slice[7..9].try_into().expect("failed to capture slice");
        let j_slice: [u8; 2] = slice[9..11].try_into().expect("failed to capture slice");
        let k_slice: [u8; 2] = slice[11..13].try_into().expect("failed to capture slice");
        let real_slice: [u8; 2] = slice[13..15].try_into().expect("failed to capture slice");
        let i = u16::from_le_bytes(i_slice);
        let j = u16::from_le_bytes(j_slice);
        let k = u16::from_le_bytes(k_slice);
        let real = u16::from_le_bytes(real_slice);
        let total = (
            i as f32 / 16384.0,
            j as f32 / 16384.0,
            k as f32 / 16384.0,
            real as f32 / 16384.0,
        );
        info!("{}", total);
        total
        // } else {
        // (0, 0, 0, 0)
        // }
    }

    fn increment_seq_num(&mut self, read_write: bool, channel: u8, seq_num: Option<u8>) -> u8 {
        if channel < 6 {
            match read_write {
                true => {
                    let temp = self.seq_num_w[channel as usize] as u16;
                    self.seq_num_w[channel as usize] = ((temp + 1) % 256) as u8;
                    ((temp + 1) % 256) as u8
                }
                false => {
                    if let Some(num) = seq_num {
                        self.seq_num_r[channel as usize] = num;
                        num
                    } else {
                        let temp = self.seq_num_r[channel as usize] as u16;
                        self.seq_num_r[channel as usize] = ((temp + 1) % 256) as u8;
                        ((temp + 1) % 256) as u8
                    }
                }
            }
        } else {
            0
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MyError {
    Placeholder, // Add other errors for your driver here.
}
