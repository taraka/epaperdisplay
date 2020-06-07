

extern {
    fn DEV_Module_Init() -> u8;
    fn DEV_Delay_ms(xms: u32);
    fn DEV_Module_Exit();
}


pub fn module_init() -> Result<(), u8> {
    match unsafe { DEV_Module_Init() } {
        0 => Ok(()),
        e => Err(e)
    }
}

pub fn module_exit() {
    unsafe { DEV_Module_Exit() }
}

pub fn delay_ms(delay: u32) {
    unsafe { DEV_Delay_ms(delay) }
}