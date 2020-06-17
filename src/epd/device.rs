use std::{thread, time};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::gpio::{Gpio, OutputPin, InputPin, Level};


pub struct Pi {
    spi: Spi,
    reset_pin: OutputPin,
    dc_pin: OutputPin,
    cs_pin: OutputPin,
    busy_pin: InputPin
}

pub enum Pin {
    ResetPin = 17,
    DcPin = 25,
    CsPin = 8,
    BusyPin = 24
}

impl Pi {
    pub fn init() -> Pi {
        let pi = Pi {
            spi : Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).expect("Failed to start SPI bus"),
            reset_pin: Gpio::new().unwrap().get(Pin::ResetPin as u8).unwrap().into_output(),
            dc_pin: Gpio::new().unwrap().get(Pin::DcPin as u8).unwrap().into_output(),
            cs_pin: Gpio::new().unwrap().get(Pin::CsPin as u8).unwrap().into_output(),
            busy_pin: Gpio::new().unwrap().get(Pin::BusyPin as u8).unwrap().into_input()
        };

        return pi;
    }

    // Not sure why this function was on the pi module
    pub fn delay_ms(&self, delay: u64) {
        thread::sleep(time::Duration::from_millis(delay));
    }

    pub fn write(&mut self, pin_id: Pin, value: bool) {
        let pin = match pin_id {
            Pin::DcPin => &mut self.dc_pin,
            Pin::ResetPin => &mut self.reset_pin,
            Pin::CsPin => &mut self.cs_pin,
            Pin::BusyPin => panic!("Can't call write on the busy pin")
        };

        pin.write(if value { Level::High } else { Level::Low });
    }

    pub fn read(&mut self, pin_id: Pin) -> bool {
        let pin = match pin_id {
            Pin::BusyPin => &self.busy_pin,
            _ => panic!("Cant write to that pin")
        };

        pin.read() == Level::High
    }

    pub fn spi_write_byte(&mut self, value: u8) {
        self.spi.write(&[value]).unwrap();
    }
}