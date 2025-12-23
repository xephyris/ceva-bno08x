#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;
use embedded_hal::{self as hal};
use heapless::Vec;
use panic_probe as _;

use hal::digital::InputPin;
use hal::spi::*;

use defmt::*;

use crate::config::DEFAULT_REPORT_INTERVAL;
use crate::data::{Packet, ProductId, VarBuf};
use crate::parsing::{get_feature_dependencies, get_report_length};
use crate::register::*;
use crate::sensors::Sensors;

mod config;
pub mod data;
pub mod error;
mod parsing;
pub mod register;
mod sensors;

const WRITE: bool = true;
const READ: bool = false;
const TIMEOUT: u32 = 2000000;

// BAUD RATE MUST BE 100000 HZ AT 3MHZ SPI FREQUENCY!!!!!!
pub struct BNO08x<I2C, D> {
    address: u8,
    i2c: I2C,
    delay: D,
    seq_num_w: [u8; 6],
    seq_num_r: [u8; 6],
    sensors: Sensors,
    features: Vec<ReportId, 42>,
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
            sensors: Sensors::new(),
            features: Vec::new(),
        }
    }

    fn read_header(&mut self) -> Packet {
        let mut header = [0u8; 4];
        self.i2c.read(self.address, &mut header).ok();

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
        self.delay.delay_ms(5);
        // info!("R PACK LENGTH: {}", out.packet_length());
        // info!("R CHANNEL NUM: {}", 0);
        // info!("R SEQ NUM: {}", out.seq_num());
        // info!(
        // "R CHANNEL {} HAS {} BYTES AVAILABLE",
        // out.channel(),
        // out.data_length()
        // );
        self.seq_num_r[0 as usize] = out.seq_num();

        if out.packet_length() == 0 {
            // info!("NO PACKET AVAILABLE");
        }
        self.i2c.read(self.address, out.as_mut_data(true)).ok();

        out
    }

    pub fn send_packet(&mut self, channel: u8, data: &[u8]) {
        let seq = self.increment_seq_num(WRITE, channel, None);
        let mut write = Packet::from_data_buf(data, channel, seq, false).expect("PacketGen failed");

        debug!("Packet Created");
        self.i2c
            .write(self.address, write.full_packet().as_slice())
            .ok();
        debug!("PACKET SENT");
    }

    fn wait_for_packeet(
        &mut self,
        channel: u8,
        report_id: Option<SH2Read>,
        max: Option<u8>,
    ) -> Packet {
        let max_attempts = max.unwrap_or(0);

        let mut out = Packet::new(true);
        if max_attempts > 0 {
            let mut retries = 0;
            if let Some(report_id) = report_id {
                while out.channel() != channel
                    && out.report_id() != Register::Read(report_id).addr()
                    && retries < max_attempts
                {
                    retries += 1;
                    out = self.read_packet();
                }
            } else {
                while out.channel() != channel && retries < max_attempts {
                    retries += 1;
                    out = self.read_packet();
                }
            }
            // info!("FEATURE REQUEST RESPONSE: {:#X}", out.as_mut_data(true));
        } else {
            if let Some(report_id) = report_id {
                while out.channel() != channel
                    && out.report_id() != Register::Read(report_id).addr()
                {
                    out = self.read_packet();
                }
            } else {
                while out.channel() != channel {
                    out = self.read_packet();
                }
            }
            // info!("FEATURE REQUEST RESPONSE: {:#X}", out.as_mut_data(true));
        }

        out
    }

    pub fn read_product_id(&mut self) -> Result<bool, MyError> {
        debug!("READING P ID");
        let mut buf_data = [Register::Write(SH2Write::ProductIDRequest).addr(), 0x00];
        self.send_packet(2, &buf_data);
        // self.increment_seq_num(WRITE, 2, None);

        let mut retries = 0;
        // let max = 3;
        let mut out = self.wait_for_packeet(2, Some(SH2Read::ProductIDResponse), None);

        let product_id = ProductId::new(out.as_mut_data(false));
        // info!("{:?}", product_id.display());

        if product_id.display().0 != (0, 0) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn enable_features(
        &mut self,
        feature_id: ReportId,
        interval: Option<u32>,
        sens_specific: Option<u32>,
    ) {
        // if !self.features.contains(&feature_id) {
        let mut data_buffer = [0_u8; 17];

        if feature_id == ReportId::PersonalActClassifier {
            debug!("Unimplemented");
        } else {
            data_buffer[0] = 0xFD;
            data_buffer[1] = feature_id as u8;
            data_buffer[5..9].copy_from_slice(&u32::to_le_bytes(
                interval.unwrap_or(DEFAULT_REPORT_INTERVAL),
            ));
            data_buffer[13..17].copy_from_slice(&u32::to_le_bytes(sens_specific.unwrap_or(0)));

            let deps = get_feature_dependencies(feature_id);
            if deps.len() > 0 {
                for dep in deps {
                    if !self.features.contains(dep) {
                        self.enable_features(*dep, None, None);
                    }
                }
            }
            warn!("ENABLE FEATURES OUTPUT: {}", &data_buffer);
            self.send_packet(2, &data_buffer);

            debug!("PACKET FAILED");
            let mut retries = 0;
            let mut out = Packet::new(true);
            while out.channel() != 2
                && out.report_id() != Register::Read(SH2Read::GetFeatureResponse).addr()
            {
                info!(
                    "OUT CHANNEL IS {} \n OUT REPORT ID IS {:#X}",
                    out.channel(),
                    out.report_id()
                );
                retries += 1;
                out = self.read_packet();
            }
            // self.features.push(feature_id).ok();
        }
        // }
    }

    pub fn quaternions(&mut self) -> (f32, f32, f32, f32) {
        // info!("READING QUATERNIONS");
        self.delay.delay_ms(2);
        let mut out = self.wait_for_packeet(3, Some(SH2Read::GetFeatureResponse), None);
        if out.data_length() > 5 {
            self.parse_sensor_report(out);
            self.sensors.quaternions
        } else {
            (0.0, 0.0, 0.0, 0.0)
        }
    }

    fn parse_sensor_report(&mut self, mut out: Packet) {
        let mut data = out.as_mut_data(false);
        let timestamping: &[u8] = &data[0..5];
        let mut index = 5;
        let max = data.len().checked_sub(15).unwrap_or(2);
        let mut attempts = 0;
        while index < data.len() && attempts < max {
            if let Some((id, length)) = get_report_length(data[index]) {
                match id {
                    ReportId::AccelerometerCalibrated => {
                        self.sensors
                            .update_data(id, &data[(index + 4)..(index + length as usize)]);
                        info!("ACCELERATION NEW {}", self.sensors.acceleration);
                    }
                    ReportId::RotationVector => {
                        // let position =
                        // self.parse_quaternions(&data[index..(index + length as usize)]);
                        self.sensors
                            .update_data(id, &data[(index + 4)..(index + length as usize)]);
                        info!("QUATERNIONS NEW {}", self.sensors.quaternions);
                    }
                    _ => debug!("Unimplemented"),
                }
                index += length as usize;
            }
            attempts += 1;
        }
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
