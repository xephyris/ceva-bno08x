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

#[derive(Debug)]
pub struct BNO08x<SPI, D, IP, WP, RP> {
    spi: SPI,
    interrupt: IP,
    wake: WP,
    reset: RP,
    delay: D,
    irq_time: u32,
    seq_num_w: [u32; 6],
    seq_num_r: [u32; 6],
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
            irq_time: 5,
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

    pub fn read_header(&mut self) {
        self.wait_for_interrupt();
        let mut header = [0; 4];
        self.spi.transfer_in_place(&mut header).ok();

        info!("{:X} ", header);
    }

    pub fn read_product_id(&mut self) -> Result<(), MyError<SPI>> {
        let buf = [Register::Write(SH2Write::ProductIDRequest).addr()];
        info!("Sending buffer {}", &buf);
        let mut out = Packet::new();
        self.wait_for_interrupt();
        self.spi
            .transaction(&mut [Operation::Write(&buf), Operation::Read(out.as_mut_header())])
            .ok();
        out.process_header();
        self.spi
            .transaction(&mut [Operation::Write(&buf), Operation::Read(out.as_mut_data())])
            .ok();
        info!("{:X}", out.as_mut_data());
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MyError<SPI> {
    Spi(SPI),
    // Add other errors for your driver here.
}
