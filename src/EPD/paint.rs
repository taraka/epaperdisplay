use std::ffi::CString;
use std::os::raw::c_char;

extern {
    fn Paint_NewImage(image: *mut u8, width: u16, height: u16, rotate: u16, color: u16);
    fn Paint_SelectImage(image: *mut u8);
    fn Paint_Clear(color: u16);
    fn Paint_DrawBitMap(image: *const u8);


    fn Paint_DrawPoint(x_point: u16, y_point: u16, color: u16, dot_pixel: Dot_Pixel, dot_style: Dot_Style);
    fn Paint_DrawLine(x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: u16, line_width: Dot_Pixel, line_style: Line_Style);
    fn Paint_DrawRectangle(x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: u16, stroke_width: Dot_Pixel, draw_fill: Draw_Fill);
    fn Paint_DrawCircle(x_center: u16, y_center: u16, radius: u16, color: u16, stroke_width: Dot_Pixel, draw_fill: Draw_Fill);
    fn Paint_DrawString_EN(x_start: u16, y_start: u16, string: *const c_char, font: *const Font, fg_color: u16, bg_color: u16);

}

extern "C" {
    pub static Font24: Font;
    pub static Font20: Font;
    pub static Font16: Font;
    pub static Font12: Font;
    pub static Font8: Font;
}

pub type Image = Box<[u8]>;

pub enum Color {
    White = 0xff,
    Black = 0x00
}

#[repr(C)]
pub enum Dot_Pixel {
    DOT_PIXEL_1X1  = 1,
    DOT_PIXEL_2X2,
    DOT_PIXEL_3X3,
    DOT_PIXEL_4X4,
    DOT_PIXEL_5X5,
    DOT_PIXEL_6X6,
    DOT_PIXEL_7X7,
    DOT_PIXEL_8X8,
}
#[repr(C)]
pub enum Dot_Style {
    DOT_FILL_AROUND  = 1,		// dot pixel 1 x 1
    DOT_FILL_RIGHTUP  , 		// dot pixel 2 X 2
}

#[repr(C)]
pub enum Line_Style {
    LINE_STYLE_SOLID = 0,
    LINE_STYLE_DOTTED,
}

#[repr(C)]
pub enum Draw_Fill {
    DRAW_FILL_EMPTY = 0,
    DRAW_FILL_FULL,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Font
{
    table: *const u8,
    Width: u16,
    Height: u16
}

pub fn new_image(width: u16, height: u16) -> Image {
    let image_size: usize = ( if width % 8 == 0 { width / 8 } else { width / 8 + 1} ) as usize * height as usize;
    let mut image: Image = vec![0; image_size].into_boxed_slice();

    unsafe { Paint_NewImage(image.as_mut_ptr(), width, height, 0, Color::White as u16) };

    return image;
}

pub fn select_image(image: &mut Image) {
    unsafe { Paint_SelectImage(image.as_mut_ptr()) }
}

pub fn clear(color: Color) {
    unsafe { Paint_Clear(color as u16) }
}

pub fn draw_bitmap(image: Box<[u8]>) {
    unsafe { Paint_DrawBitMap(image.as_ptr()) }
}

pub fn draw_point(x_point: u16, y_point: u16, color: Color, dot_pixel: Dot_Pixel, dot_style: Dot_Style) {
    unsafe { Paint_DrawPoint(x_point, y_point, color as u16, dot_pixel,  dot_style); }
}

pub fn draw_line(x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: Color, line_width: Dot_Pixel, line_style: Line_Style) {
    unsafe { Paint_DrawLine(x_start, y_start, x_end, y_end, color as u16, line_width, line_style) }
}

pub fn draw_rectangle(x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: Color, line_width: Dot_Pixel, draw_fill: Draw_Fill) {
    unsafe { Paint_DrawRectangle(x_start, y_start, x_end, y_end, color as u16, line_width, draw_fill) }
}

pub fn draw_circle(x_center: u16, y_center: u16, radius: u16, color: Color, line_width: Dot_Pixel, draw_fill: Draw_Fill) {
    unsafe { Paint_DrawCircle(x_center, y_center, radius, color as u16, line_width, draw_fill) }
}

pub fn draw_string(x_start: u16, y_start: u16, string: String, font: Box<Font>, fg_color: Color, bg_color: Color) {
    unsafe { Paint_DrawString_EN(x_start, y_start, CString::new(string).expect("failed to make string").as_ptr(), &*font, fg_color as u16, bg_color as u16) }
}


pub fn font8() -> Box<Font> {
    return unsafe { Box::new(Font8) };
}
pub fn font12() -> Box<Font> {
    return unsafe { Box::new(Font12) };
}
pub fn font16() -> Box<Font> {
    return unsafe { Box::new(Font16) };
}
pub fn font20() -> Box<Font> {
    return unsafe { Box::new(Font20) };
}
pub fn font24() -> Box<Font> {
    return unsafe { Box::new(Font24) };
}