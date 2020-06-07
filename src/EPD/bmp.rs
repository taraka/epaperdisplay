use std::ffi::CString;
use std::os::raw::c_char;

extern {
    fn GUI_ReadBmp(path: *const c_char, x_start: u16, y_start: u16);
}

pub fn read_bmp(path: String, x_start: u16, y_start: u16) {
    unsafe { GUI_ReadBmp(CString::new(path).expect("failed to make string").as_ptr() , x_start, y_start) }
}
