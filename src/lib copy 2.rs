// #![no_std]
// use core::convert::Infallible;

// use embedded_hal::digital::OutputPin;
// use embedded_hal::{self as hal, delay};
// use panic_probe as _;

// use hal::digital::InputPin;
// use hal::spi::*;

// use defmt::*;

// use crate::error::CSPinError;

// use crate::register::*;

// pub mod error;
// mod register;
// // pub struct BNO08x<SPI, CS> {
// //     spi: SPI,
// //     cs: CS,
// // }

// // impl<SPI, CS> BNO08x<SPI, CS>
// // where
// //     SPI: SpiDevice,
// //     CS: OutputPin,
// // {

// pub struct BNO08x<'a, WP, RP> {
//     spi: &'a mut dyn SpiDevice<Error = Infallible>,
//     wake: WP,
//     reset: RP,
// }

// impl<'a, WP, RP> BNO08x<'a, WP, RP>
// where
//     WP: OutputPin,
//     RP: OutputPin,
// {
//     pub fn new(spi: &'a mut (dyn SpiDevice<Error = Infallible>), wake: WP, reset: RP) -> Self {
//         Self { spi, wake, reset }
//     }

//     // Enable the sensor by Chip Select Pin
//     // pub fn select(&mut self) {
//     //     let _ = self.cs.set_low();
//     // }

//     // pub fn deselect(&mut self) {
//     //     let _ = self.cs.set_high();
//     // }

//     pub fn reset_device(&mut self) {
//         info!("BNO08x Device Reset");
//         self.reset.set_low().ok();
//         self.reset.set_high().ok();
//     }

//     pub fn send_wake(&mut self) {
//         info!("BNO08x Device Wake signal sent");
//         self.wake.set_low().ok();
//         self.wake.set_high().ok();
//     }

//     pub fn read_product_id(
//         &mut self,
//     ) -> Result<[u8; 100], MyError<&'a dyn SpiDevice<Error = Infallible>>> {
//         let buf = [Register::Write(SH2Write::ProductIDRequest).addr(), 0];
//         info!("Sending buffer {}", &buf);
//         let mut header = [0; 100];
//         self.spi
//             .transaction(&mut [Operation::Write(&buf), Operation::Read(&mut header)])
//             .ok();
//         info!("{}", header);
//         Ok(header)
//     }

//     // (buf length - shtp, channel, seqnum)
//     // pub fn read_shtp_header(buf: &mut [u8]) -> (u16, u8, u32) {
//     //     let length;
//     //     let channel;
//     //     let seqnum;

//     // }

//     // pub fn read_register(&mut self, register: u8, buffer: &mut [u8]) {}

//     // pub fn write_register(&mut self, register: u8, value: u8) {
//     //     let mut bytes = [register, 0];
//     //     self.cs.set_low().ok();
//     //     self.spi.write(&mut bytes).ok();
//     //     self.cs.set_high().ok();
//     // }

//     // pub fn read_foo(&mut self) -> Result<[u8; 2], MyError<SPI::Error>> {
//     //     let mut buf = [0; 2];

//     //     // `transaction` asserts and deasserts CS for us. No need to do it manually!
//     //     self.spi
//     //         .transaction(&mut [Operation::Write(&[0x90]), Operation::Read(&mut buf)])
//     //         .map_err(MyError::Spi)?;

//     //     Ok(buf)
//     // }
// }

// #[derive(Copy, Clone, Debug)]
// pub enum MyError<SPI> {
//     Spi(SPI),
//     // Add other errors for your driver here.
// }

#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::{self as hal, delay};
use panic_probe as _;

use hal::digital::InputPin;
use hal::spi::*;

use defmt::*;

use crate::error::CSPinError;

use crate::register::*;

pub mod error;
mod register;
// pub struct BNO08x<SPI, CS> {
//     spi: SPI,
//     cs: CS,
// }

// impl<SPI, CS> BNO08x<SPI, CS>
// where
//     SPI: SpiDevice,
//     CS: OutputPin,
// {
#[derive(Debug)]
pub struct BNO08x<SPI, D, IP, WP, RP> {
    spi: SPI,
    interrupt: IP,
    wake: WP,
    reset: RP,
    delay: D,
    int_high_count: u32,
    irq_time: u32,
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
            int_high_count: 0,
            irq_time: 5,
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
            int_high_count: 0,
            irq_time,
        }
    }

    // Enable the sensor by Chip Select Pin
    // pub fn select(&mut self) {
    //     let _ = self.cs.set_low();
    // }

    // pub fn deselect(&mut self) {
    //     let _ = self.cs.set_high();
    // }

    fn wait_for_interrupt(&mut self) {
        let mut elapsed = 0;
        let mut counted = false;
        while elapsed <= 3_000_000_000u32 / self.irq_time {
            elapsed += 1;
            if !counted {
                if self.interrupt.is_high().expect("Cannot read Pin") {
                    self.int_high_count += 1;
                }
                counted = true;
            }
            if self.interrupt.is_low().expect("Failed to read INT pin.") {
                elapsed -= 1;
                info!("{} / {}", elapsed, 3_000_000_000u32 / self.irq_time);
                info!("{}", self.int_high_count);
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
        self.wait_for_interrupt();
        self.read_header();
        info!("BNO08x Device Reset");
    }

    pub fn send_wake(&mut self) {
        info!("BNO08x Device Wake signal sent");
        self.wake.set_low().ok();
        self.wait_for_interrupt();
        self.wake.set_high().ok();
    }

    // pub fn handle_interrupt(&mut self) {
    //     let mut len_buf = [1u8; 2];
    //     self.spi.transfer(&mut len_buf, &[0x00]).unwrap();
    //     let packet_len = u16::from_le_bytes(len_buf);
    //     info!("interrupt handled");
    //     if packet_len > 0 {
    //         let mut packet = [1u8; 4];
    //         self.spi.transfer(&mut packet, &[0x00]).unwrap();

    //         // SHTP header is bytes 0..4 of packet
    //         let shtp_len = u16::from_le_bytes([packet[0], packet[1]]);
    //         let channel = packet[2];
    //         let sequence = packet[3];

    //         info!(
    //             "SHTP header: len={}, channel={}, seq={}",
    //             shtp_len, channel, sequence
    //         );
    //     }
    // }

    pub fn read_header(&mut self) {
        self.wait_for_interrupt();
        let mut header = [0; 4];
        self.spi.transfer_in_place(&mut header).ok();

        info!("{:X} ", header);
    }

    pub fn read_product_id(&mut self) -> Result<[u8; 255], MyError<SPI>> {
        let buf = [Register::Write(SH2Write::ProductIDRequest).addr(), 0];
        info!("Sending buffer {}", &buf);
        let mut r_buf = [0; 255];
        self.spi
            .transaction(&mut [Operation::Write(&buf), Operation::Read(&mut r_buf)])
            .ok();
        info!("{:X}", r_buf);
        Ok(r_buf)
    }

    // (buf length - shtp, channel, seqnum)
    // pub fn read_shtp_header(buf: &mut [u8]) -> (u16, u8, u32) {
    //     let length;
    //     let channel;
    //     let seqnum;

    // }

    // pub fn read_register(&mut self, register: u8, buffer: &mut [u8]) {}

    // pub fn write_register(&mut self, register: u8, value: u8) {
    //     let mut bytes = [register, 0];
    //     self.cs.set_low().ok();
    //     self.spi.write(&mut bytes).ok();
    //     self.cs.set_high().ok();
    // }

    // pub fn read_foo(&mut self) -> Result<[u8; 2], MyError<SPI::Error>> {
    //     let mut buf = [0; 2];

    //     // `transaction` asserts and deasserts CS for us. No need to do it manually!
    //     self.spi
    //         .transaction(&mut [Operation::Write(&[0x90]), Operation::Read(&mut buf)])
    //         .map_err(MyError::Spi)?;

    //     Ok(buf)
    // }
}

#[derive(Copy, Clone, Debug)]
pub enum MyError<SPI> {
    Spi(SPI),
    // Add other errors for your driver here.
}
