#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
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
            irq_time: 100,
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

    fn wait_for_interrupt(&mut self) -> u32 {
        let mut elapsed = 0;
        while elapsed <= 300_000_000u32 / self.irq_time {
            elapsed += 1;
            if self.interrupt.is_low().expect("Failed to read INT pin.") {
                elapsed -= 1;
                info!("{} / {}", elapsed, 3_000_000_000u32 / self.irq_time);
                info!("Interrupt recieved breaking!");
                break;
            }
            self.delay.delay_ns(self.irq_time);
        }
        if elapsed >= 300_000_000u32 / self.irq_time {
            info!("Hard Resetting");
            self.hard_reset_device();
        }
        elapsed
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

    pub fn soft_reset_device(&mut self) {
        for packet in 0..3 {
            info!("FLUSHING PACKETS");
            // self.wait_for_interrupt();
            self.read_packet();
        }
    }

    pub fn send_wake(&mut self) {
        info!("BNO08x Device Wake signal sent");
        self.wake.set_high().ok();
        self.wake.set_low().ok();
        info!("INT Status: {}", self.interrupt.is_high().ok());
        self.wait_for_interrupt();
        self.wake.set_high().ok();
        info!("Wake finished")
        // self.spi.transaction(&mut [Operation::Write(&[0x00])]).ok();
    }

    pub fn read_header(&mut self) {
        info!("INT Status: {}", self.interrupt.is_high().ok());
        info!("{}", self.wait_for_interrupt());
        let mut header = [0; 4];
        self.spi.transfer_in_place(&mut header).ok();

        info!("R {:#X} ", header);
    }

    pub fn read_packet(&mut self) -> Packet {
        let mut header = [0; 4];
        // info!("Transferring");
        // self.spi
        //     .transaction(&mut [Operation::Transfer(&mut header)])
        //     .ok();
        self.spi.transfer(&mut header, &[0u8; 4]).ok();
        let mut out = Packet::from_buf(&header);
        out.process_header(true);
        info!("R PACK LENGTH: {}", out.packet_length());
        info!("R CHANNEL NUM: {}", 0);
        info!("R SEQ NUM: {}", out.seq_num());
        self.seq_num_r[0 as usize] = out.seq_num();

        if out.packet_length() == 0 {
            info!("NO PACKET AVAILABLE");
        }

        info!(
            "R CHANNEL {} HAS {} BYTES AVAILABLE",
            out.channel(),
            out.data_length()
        );
        info!("INT Status: {}", self.interrupt.is_high().ok());
        info!("{}", self.wait_for_interrupt());

        // self.spi
        //     .transaction(&mut [Operation::Read(out.as_mut_header())])
        //     .ok();
        // self.increment_seq_num(false, out.as_mut_header()[2], Some(out.as_mut_header()[3]));
        // if out.data_length() < 273 {
        self.spi
            .transaction(&mut [
                Operation::TransferInPlace(&mut [0x00; 4]),
                Operation::TransferInPlace(out.as_mut_data()),
            ])
            .ok();
        if out.data_length() < 300 {
            info!("R DATA: {:#X}", out.as_mut_data());
        }
        out
    }

    pub fn send_packet(&mut self, channel: u8, data: &[u8]) {
        let seq = self.increment_seq_num(WRITE, channel, None);
        let mut write = Packet::from_data_buf(data, channel, seq).expect("PacketGen failed");
        info!(
            "SENDING PACKETS WITH CONTENTS {}",
            write.full_packet().as_slice()
        );
        let mut read: heapless::Vec<u8, 32767> = heapless::Vec::new();
        read.resize(write.packet_length() as usize, 0u8);
        info!("INT Status: {}", self.interrupt.is_high().ok());
        info!("{}", self.wait_for_interrupt());
        self.spi
            .transfer(read.as_mut(), write.full_packet().as_slice())
            .ok();
        info!("PACKET SENT");
    }

    pub fn wait_for_packet_type(&mut self, channel: u8, report_id: Option<u8>) -> Packet {
        info!(
            "Wating for packet on channel {} with report_id {}",
            channel,
            report_id.unwrap_or(0xFF)
        );
        let mut new_packet = self.wait_for_packet().expect("Packet failed");
        // info!("REPORT ID {}", new_packet.as_mut_data());
        loop {
            info!("NEW PACKET CHANNEL: {}", new_packet.channel());
            if new_packet.channel() == channel {
                // info!("PACKET ID NEW {}", new_packet.channel());
                if let Some(id) = report_id {
                    info!("PACKET ID {}", new_packet.report_id());
                    if new_packet.report_id() == id {
                        return new_packet;
                    } else {
                        return Packet::new();
                    }
                } else {
                    return new_packet;
                }
            } else {
                return Packet::new();
            }
        }
        return Packet::new();
    }

    pub fn read_product_id(&mut self) -> Result<(), MyError> {
        // let mut buf = Packet::from_data_buf(
        //     &[Register::Write(SH2Write::ProductIDRequest).addr(), 0x00],
        //     2,
        //     self.seq_num_w[2],
        // )
        // .expect("W Packet Channel Invalid");
        info!("READING P ID");
        let mut buf_data = [Register::Write(SH2Write::ProductIDRequest).addr(), 0x00];
        self.send_packet(2, &buf_data);
        let mut out =
            self.wait_for_packet_type(2, Some(Register::Read(SH2Read::ProductIDResponse).addr()));
        // info!("R PID {:#X}", out.full_packet().as_slice());
        info!("R PID RESPONSE: {:#X}", out.as_mut_data());
        let product_id = ProductId::new(out.as_mut_data());
        info!("{:?}", product_id.display());
        Ok(())
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

    fn wait_for_packet(&mut self) -> Result<Packet, MyError> {
        let mut new_packet;
        // info!("FETCHING PACKETE");
        info!("{}", self.wait_for_interrupt());
        new_packet = self.read_packet();
        // info!("PACKET FROM WAIT {:#X}", new_packet.as_mut_data());
        return Ok(new_packet);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MyError {
    Placeholder, // Add other errors for your driver here.
}
