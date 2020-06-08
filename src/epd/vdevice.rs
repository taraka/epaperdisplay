
use std::{thread, time};


pub fn module_init() -> Result<(), u8> {
    Ok(())
}

pub fn module_exit() {

}

pub fn delay_ms(delay: u64) {
    thread::sleep(time::Duration::from_millis(delay));
}