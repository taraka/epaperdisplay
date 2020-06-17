use std::{thread, time};
use rppal::spi::{Bus, Mode, Segment, SlaveSelect, Spi};
use rppal::gpio::{Gpio, OutputPin, InputPin, Level};


pub struct Pi {
    spi: Spi,
    reset_pin: OutputPin,
    dc_pin: OutputPin,
    cs_pin: OutputPin,
    busy_pin: InputPin
}

pub enum Pin {
    EPD_RST_PIN     = 17,
    EPD_DC_PIN      = 25,
    EPD_CS_PIN      = 8,
    EPD_BUSY_PIN    = 24
}

impl Pi {
    pub fn init() -> Pi {
        let pi = Pi {
            spi : Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).expect("Failed to start SPI bus"),
            reset_pin: Gpio::new().unwrap().get(Pin::EPD_RST_PIN as u8).unwrap().into_output(),
            dc_pin: Gpio::new().unwrap().get(Pin::EPD_DC_PIN as u8).unwrap().into_output(),
            cs_pin: Gpio::new().unwrap().get(Pin::EPD_CS_PIN as u8).unwrap().into_output(),
            busy_pin: Gpio::new().unwrap().get(Pin::EPD_BUSY_PIN as u8).unwrap().into_input()
        };

        return pi;
    }

    pub fn exit(&self) {

    }

    // Not sure why this function was on the pi module
    pub fn delay_ms(&self, delay: u64) {
        thread::sleep(time::Duration::from_millis(delay));
    }

    pub fn write(&mut self, pin_id: Pin, value: bool) {
        let mut pin = match pin_id {
            Pin::EPD_DC_PIN => &mut self.dc_pin,
            Pin::EPD_RST_PIN => &mut self.reset_pin,
            Pin::EPD_CS_PIN => &mut self.cs_pin,
            Pin::EPD_BUSY_PIN => panic!("Can't call write on the busy pin")
        };

        pin.write(if value { Level::High } else { Level::Low });
    }

    pub fn read(&mut self, pin_id: Pin) -> bool {
        let mut pin = match pin_id {
            Pin::EPD_BUSY_PIN => &self.busy_pin,
            _ => panic!("Cant write to that pin")
        };

        pin.read() == Level::High
    }

    pub fn spi_write_byte(&mut self, value: u8) {
        self.spi.write(&[value]);
    }
}