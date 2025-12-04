#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::{self as hal};
use panic_probe as _;

use hal::digital::InputPin;
use hal::spi::*;

use defmt::*;

use crate::data::{Packet, VarBuf};
use crate::register::*;

pub mod data;
pub mod error;
mod register;

// BAUD RATE MUST BE 100000 HZ AT 3MHZ SPI FREQUENCY!!!!!!
#[derive(Debug)]
pub struct BNO08x<SPI, D, IP, WP, RP> {
    spi: SPI,
    interrupt: IP,
    wake: WP,
    reset: RP,
    delay: D,
    irq_time: u32,
    seq_num_w: [u8; 6],
    seq_num_r: [u8; 6],
}

impl<SPI, D, IP, WP, RP> BNO08x<SPI, D, IP, WP, RP>
where
    SPI: SpiDevice,
    IP: InputPin,
    WP: OutputPin,
    RP: OutputPin,
    D: DelayNs,
{
    pub fn new(spi: SPI, interrupt: IP, wake: WP, reset: RP, delay: D) -> Self {
        Self {
            spi,
            interrupt,
            wake,
            reset,
            delay,
            irq_time: 2,
            seq_num_w: [0; 6],
            seq_num_r: [0; 6],
        }
    }

    pub fn custom_interrupt(
        spi: SPI,
        interrupt: IP,
        wake: WP,
        reset: RP,
        delay: D,
        irq_time: u32,
    ) -> Self {
        Self {
            spi,
            interrupt,
            wake,
            reset,
            delay,
            irq_time,
            seq_num_w: [0; 6],
            seq_num_r: [0; 6],
        }
    }

    fn wait_for_interrupt(&mut self) {
        let mut elapsed = 0;
        while elapsed <= 3_000_000_000u32 / self.irq_time {
            elapsed += 1;
            if self.interrupt.is_low().expect("Failed to read INT pin.") {
                // self.delay.delay_ms(1000);
                elapsed -= 1;
                info!("{} / {}", elapsed, 3_000_000_000u32 / self.irq_time);
                info!("Interrupt recieved breaking!");
                break;
            }
            self.delay.delay_ns(self.irq_time);
        }
        if elapsed == 3_000_000_000u32 / self.irq_time {
            info!("Hard Resetting");
            self.hard_reset_device();
        }
    }

    pub fn hard_reset_device(&mut self) {
        info!("BNO08x Device Resetting");
        self.reset.set_high().ok();
        self.delay.delay_ms(10);
        self.reset.set_low().ok();
        self.delay.delay_ms(10);
        self.reset.set_high().ok();
        // self.delay.delay_ms(30);
        self.send_wake();
        self.read_packet();
        info!("BNO08x Device Reset");
    }

    pub fn send_wake(&mut self) {
        info!("BNO08x Device Wake signal sent");
        let mut fill_buf = [0u8; 600];
        self.wake.set_high().ok();
        self.wake.set_low().ok();
        self.wait_for_interrupt();
        self.wake.set_high().ok();
        self.spi
            .transaction(&mut [Operation::Read(&mut fill_buf)])
            .ok();
        info!("INIT READ {:#X}", &fill_buf);
    }

    pub fn read_header(&mut self) {
        self.wait_for_interrupt();
        let mut header = [0; 4];
        self.spi.transfer_in_place(&mut header).ok();

        info!("{:#X} ", header);
    }

    pub fn read_packet(&mut self) {
        let mut header = [0u8; 4];
        let mut payload = [0u8; 272]; // max SHTP packet size

        // One transaction: CS low across both ops
        self.spi
            .transaction(&mut [
                Operation::TransferInPlace(&mut header),
                Operation::TransferInPlace(&mut payload),
            ])
            .unwrap();

        // Parse header
        info!("HEADER {:#X}", &header);
        let len = u16::from_le_bytes([header[0], header[1]]) as usize;
        let channel = header[2];
        let seq = header[3];
        let data_len;
        if len < 4 || len > 272 {
            // Misframed → resync/reset
            data_len = 0;
        } else {
            data_len = len - 4;
        }
        let data = &payload[..data_len];

        // Process packet
        println!(
            "len={}, channel={}, seq={}, data={:#X}",
            len, channel, seq, data
        );
    }

    // pub fn read_packet(&mut self) {
    //     let mut out = Packet::new();
    //     self.wait_for_interrupt();
    //     self.spi
    //         .transaction(&mut [Operation::Read(out.as_mut_header())])
    //         .ok();
    //     self.increment_seq_num(false, out.as_mut_header()[2], Some(out.as_mut_header()[3]));
    //     out.process_header(true);
    //     self.spi
    //         .transaction(&mut [Operation::TransferInPlace(out.as_mut_data())])
    //         .ok();
    //     info!("{:#X}", out.as_mut_data());
    // }

    // pub fn read_packet(&mut self) {
    //     let mut header = [0; 4];
    //     self.spi
    //         .transaction(&mut [Operation::Transfer(&mut header, &mut [0u8; 4])])
    //         .ok();
    //     let mut out = Packet::from_buf(&header);
    //     out.process_header(true);
    //     info!("PACK LENGTH: {}", out.packet_length());
    //     info!("CHANNEL NUM: {}", 0);
    //     info!("SEQ NUM: {}", out.seq_num());
    //     self.seq_num_r[0 as usize] = out.seq_num();

    //     if out.packet_length() == 0 {
    //         info!("NO PACKET AVAILABLE");
    //     }

    //     info!(
    //         "CHANNEL {} HAS {} BYTES AVAILABLE",
    //         out.channel(),
    //         out.data_length()
    //     );
    //     self.wait_for_interrupt();
    //     // self.spi
    //     //     .transaction(&mut [Operation::Read(out.as_mut_header())])
    //     //     .ok();
    //     // self.increment_seq_num(false, out.as_mut_header()[2], Some(out.as_mut_header()[3]));
    //     // if out.data_length() < 273 {
    //     self.spi
    //         .transaction(&mut [Operation::TransferInPlace(out.as_mut_data())])
    //         .ok();
    //     info!("DATA: {:#X}", out.as_mut_data());

    //     // let mut header = [0; 4];
    //     // self.spi
    //     //     .transaction(&mut [Operation::Transfer(&mut header, &mut [0u8; 4])])
    //     //     .ok();
    //     // let mut out = Packet::from_buf(&header);
    //     // out.process_header(true);
    //     // info!("PACK LENGTH 2 : {}", out.packet_length());
    //     // info!("CHANNEL NUM2 : {}", 0);
    //     // info!("SEQ NUM 2 : {}", out.seq_num());
    //     // self.seq_num_r[0 as usize] = out.seq_num();

    //     // if out.packet_length() == 0 {
    //     //     info!("NO PACKET AVAILABLE 2 ");
    //     // }

    //     // info!(
    //     //     "CHANNEL {} HAS {} BYTES AVAILABLE 2",
    //     //     out.channel(),
    //     //     out.data_length()
    //     // );

    //     // // self.spi
    //     // //     .transaction(&mut [Operation::Read(out.as_mut_header())])
    //     // //     .ok();
    //     // // self.increment_seq_num(false, out.as_mut_header()[2], Some(out.as_mut_header()[3]));
    //     // if out.data_length() < 273 {
    //     //     self.spi
    //     //         .transaction(&mut [Operation::Read(out.as_mut_data())])
    //     //         .ok();
    //     //     info!("DATA 2 : {:#X}", out.as_mut_data());
    //     // }
    // }

    pub fn read_product_id(&mut self) -> Result<(), MyError<SPI>> {
        let mut buf = Packet::from_data_buf(
            &[Register::Write(SH2Write::ProductIDRequest).addr(), 0x00],
            2,
            self.seq_num_w[2],
        )
        .expect("Packet Channel Invalid");
        info!("Sending buffer {}", buf.full_packet().as_slice());
        let mut out = Packet::new();
        self.increment_seq_num(true, 2, None);
        info!(
            "FULL PACKET WITH SHTP HEADER {}",
            buf.full_packet().as_slice()
        );
        // self.wait_for_interrupt();
        self.spi
            .transaction(&mut [Operation::Write(&buf.full_packet())])
            .ok();
        self.wait_for_interrupt();
        self.spi
            .transaction(&mut [Operation::TransferInPlace(out.as_mut_header())])
            .ok();

        out.process_header(true);
        self.spi
            .transaction(&mut [Operation::TransferInPlace(out.as_mut_data())])
            .ok();
        // info!("{:#X}", out.as_mut_data());
        Ok(())
    }

    fn increment_seq_num(&mut self, read_write: bool, channel: u8, seq_num: Option<u8>) {
        if channel < 6 {
            match read_write {
                true => {
                    let temp = self.seq_num_w[channel as usize] as u16;
                    self.seq_num_w[channel as usize] = ((temp + 1) % 256) as u8;
                }
                false => {
                    if let Some(num) = seq_num {
                        self.seq_num_w[channel as usize] = num
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MyError<SPI> {
    Spi(SPI),
    // Add other errors for your driver here.
}
