use std::{thread, time};
use rppal::spi::{Bus, Mode, Segment, SlaveSelect, Spi};

extern {
    fn DEV_Module_Init() -> u8;
    fn DEV_Module_Exit();

    fn DEV_Digital_Write(pin: u16, value: u8);
    fn DEV_Digital_Read(pin: u16) -> u8;

    fn DEV_SPI_WriteByte(value: u8);
    fn DEV_SPI_Write_nByte(data: *const u8, len: u32);
}

pub enum Pin {
    EPD_RST_PIN     = 17,
    EPD_DC_PIN      = 25,
    EPD_CS_PIN      = 8,
    EPD_BUSY_PIN    = 24
}

pub fn module_init() -> Result<(), u8> {
    match unsafe { Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0) } {
        Ok(_) => Ok(()),
        e => Err(1)
    }
}

pub fn module_exit() {
    unsafe { DEV_Module_Exit() }
}

pub fn delay_ms(delay: u64) {
    thread::sleep(time::Duration::from_millis(delay));
}

pub fn digital_write(pin: Pin, value: u8) {
    unsafe { DEV_Digital_Write(pin as u16, value) }
}

pub fn digital_read(pin: Pin) -> u8 {
    unsafe { DEV_Digital_Read(pin as u16) }
}

pub fn spi_write_byte(value: u8) {
    unsafe { DEV_SPI_WriteByte(value) }
}