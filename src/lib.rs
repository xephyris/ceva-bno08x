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
pub struct BNO08x<I2C> {
    i2c: I2C,
    seq_num_w: [u8; 6],
    seq_num_r: [u8; 6],
}

impl<I2C> BNO08x<I2C> where I2C: I2c {}

#[derive(Copy, Clone, Debug)]
pub enum MyError {
    Placeholder, // Add other errors for your driver here.
}
