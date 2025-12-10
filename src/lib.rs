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
        if out.channel() != 3 && out.channel() != 0 {
            info!("R CHANNEL NUM: {}", 0);
            info!("R SEQ NUM: {}", out.seq_num());
            info!(
                "R CHANNEL {} HAS {} BYTES AVAILABLE",
                out.channel(),
                out.data_length()
            );
        }
        self.seq_num_r[0 as usize] = out.seq_num();

        if out.packet_length() == 0 {
            info!("NO PACKET AVAILABLE");
        }
        let mut spacer = [0u8; 2];
        self.i2c.read(self.address, out.as_mut_data(true)).ok();

        if out.channel() != 3 {
            info!("FULL PACKET {:#X}", out.full_packet().as_slice());
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
        println!("READ WHILE WRITING: {}", read.full_packet().as_slice());
        self.i2c
            .write_read(
                self.address,
                write.full_packet().as_slice(),
                read.as_mut_data(false),
            )
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

        let mut retries = 0;
        // let max = 3;
        let mut out = Packet::new(true);
        while out.channel() != 2 {
            retries += 1;
            out = self.read_packet();
            if out.channel() == 2 {
                info!("{}", out.full_packet().as_mut_slice());
                loop {}
            }
        }

        let product_id = ProductId::new(out.as_mut_data(false));
        info!("{:?}", product_id.display());

        if product_id.display().0 != (0, 0) {
            Ok(true)
        } else {
            Ok(false)
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
