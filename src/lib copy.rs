#![no_std]
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
pub struct BNO08x<SPI, WP, RP> {
    spi: SPI,
    wake: WP,
    reset: RP,
}

impl<SPI, WP, RP> BNO08x<SPI, WP, RP>
where
    SPI: SpiDevice,
    WP: OutputPin,
    RP: OutputPin,
{
    pub fn new(spi: SPI, wake: WP, reset: RP) -> Self {
        Self { spi, wake, reset }
    }

    // Enable the sensor by Chip Select Pin
    // pub fn select(&mut self) {
    //     let _ = self.cs.set_low();
    // }

    // pub fn deselect(&mut self) {
    //     let _ = self.cs.set_high();
    // }

    pub fn reset_device(&mut self) {
        info!("BNO08x Device Reset");
        self.reset.set_low().ok();
        self.reset.set_high().ok();
    }

    pub fn send_wake(&mut self) {
        info!("BNO08x Device Wake signal sent");
        self.wake.set_low().ok();
        self.wake.set_high().ok();
    }

    pub fn read_product_id(&mut self) -> Result<[u8; 100], MyError<SPI>> {
        let buf = [Register::Write(SH2Write::ProductIDRequest).addr(), 0];
        info!("Sending buffer {}", &buf);
        let mut header = [0; 100];
        self.spi
            .transaction(&mut [Operation::Write(&buf), Operation::Read(&mut header)])
            .ok();
        info!("{}", header);
        Ok(header)
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
