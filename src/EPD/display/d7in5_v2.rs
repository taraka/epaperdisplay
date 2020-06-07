
extern {
    fn EPD_7IN5_V2_Init();
    fn EPD_7IN5_V2_Clear();
}

pub const WIDTH: u16 = 800;
pub const HEIGHT: u16 = 480;

pub fn init() {
    unsafe { EPD_7IN5_V2_Init() }
}

pub fn clear() {
    unsafe { EPD_7IN5_V2_Clear() }
}


