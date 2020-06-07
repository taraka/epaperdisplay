use crate::epd::paint::Image;


extern {
    fn EPD_7IN5_V2_Init();
    fn EPD_7IN5_V2_Clear();
    fn EPD_7IN5_V2_Display(image: *mut u8);
    fn EPD_7IN5_V2_Sleep();
}

pub const WIDTH: u16 = 800;
pub const HEIGHT: u16 = 480;

pub fn init() {
    unsafe { EPD_7IN5_V2_Init() }
}

pub fn clear() {
    unsafe { EPD_7IN5_V2_Clear() }
}

pub fn display(image: &mut Image) {
    unsafe { EPD_7IN5_V2_Display(image.as_mut_ptr()) }
}

pub fn sleep() {
    unsafe { EPD_7IN5_V2_Sleep(); }
}

