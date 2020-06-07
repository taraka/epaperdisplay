use std::ffi::CString;
use std::os::raw::c_char;

extern {
    fn Paint_NewImage(image: *mut u8, width: u16, height: u16, rotate: u16, color: u16);
    fn Paint_SelectImage(image: *mut u8);
    fn Paint_Clear(color: u16);
    fn Paint_DrawBitMap(image: *const u8);

    fn Paint_DrawCircle(x_center: u16, y_center: u16, radius: u16, color: u16, stroke_width: Dot_Pixel, draw_fill: Draw_Fill);
    fn Paint_DrawString_EN(x_start: u16, y_start: u16, string: *const c_char, font: *const Font, fg_color: u16, bg_color: u16);
    fn Paint_DrawNum(x_start: u16, y_start: u16, string: i32, font: *const Font, fg_color: u16, bg_color: u16);

}

pub struct Image {
    pub(crate) image: ImageData,
    width: u16,
    height: u16,
    width_memory: u16,
    height_memory: u16,
    color: Color,
    rotate: Rotation,
    mirror: Mirror,
    width_byte: u16,
    height_byte: u16,
    scale: u16
}

extern "C" {
    pub static Font24: Font;
    pub static Font20: Font;
    pub static Font16: Font;
    pub static Font12: Font;
    pub static Font8: Font;
}

pub type ImageData = Box<[u8]>;

#[derive(Clone, Copy)]
pub enum Color {
    White = 0xff,
    Black = 0x00
}

enum Mirror {
    NONE  = 0,
    HORIZONTAL = 1,
    VERTICAL = 2,
    ORIGIN = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
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
pub enum Rotation {
    R0  = 0,
    R90 = 90,
    R180 = 180,
    R270 = 270
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

pub fn new_image(width: u16, height: u16, color: Color) -> Image {

    let image_size: usize = ( if width % 8 == 0 { width / 8 } else { width / 8 + 1} ) as usize * height as usize;

    Image {
        image: vec![0; image_size].into_boxed_slice(),
        width_memory: width,
        height_memory: height,
        color,
        scale: 2,
        width_byte: width / 8,
        height_byte: height,
        rotate: Rotation::R0,
        mirror: Mirror::NONE,
        width,
        height
    }
}

pub fn select_image(image: &mut ImageData) {
    unsafe { Paint_SelectImage(image.as_mut_ptr()) }
}

pub fn clear(color: Color) {
    unsafe { Paint_Clear(color as u16) }
}

pub fn draw_bitmap(image: Box<[u8]>) {
    unsafe { Paint_DrawBitMap(image.as_ptr()) }
}

pub fn draw_circle(x_center: u16, y_center: u16, radius: u16, color: Color, line_width: Dot_Pixel, draw_fill: Draw_Fill) {
    unsafe { Paint_DrawCircle(x_center, y_center, radius, color as u16, line_width, draw_fill) }
}

pub fn draw_string(x_start: u16, y_start: u16, string: String, font: Box<Font>, fg_color: Color, bg_color: Color) {
    unsafe { Paint_DrawString_EN(x_start, y_start, CString::new(string).expect("failed to make string").as_ptr(), &*font, fg_color as u16, bg_color as u16) }
}

pub fn draw_num(x_start: u16, y_start: u16, number: i32, font: Box<Font>, fg_color: Color, bg_color: Color) {
    unsafe { Paint_DrawNum(x_start, y_start, number, &*font, fg_color as u16, bg_color as u16) }
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

impl Image {

    pub fn clear(&mut self, color: Color) {
            for y in  0..self.height_byte {
                for x in 0..self.height_byte {//8 pixel =  1 byte
                    self.image[( x + y * self.width_byte) as usize] = Color;
                }
        }
    }

    pub fn draw_point(&mut self, x_point: u16, y_point: u16, color: Color, dot_pixel: Dot_Pixel, dot_style: Dot_Style) {
        if x_point > self.width || y_point > self.height {
            return;
        }

        let dot_size = dot_pixel as u16;

        if dot_style == Dot_Style::DOT_FILL_AROUND {
            for xdir_num in 0..2 * Dot_Pixel - 1 {
                for ydir_num in 0..2 * Dot_Pixel - 1 {
                    if x_point + xdir_num - dot_size < 0 || y_point + ydir_num - dot_size < 0 {
                        break;
                    }
                    // printf("x = %d, y = %d\r\n", Xpoint + XDir_Num - Dot_Pixel, Ypoint + YDir_Num - Dot_Pixel);
                    self.set_pixel(x_point + xdir_num - dot_size, y_point + ydir_num - dot_size, Color);
                }
            }
        } else {
            for xdir_num in  0..dot_size {
                for ydir_num in 0..dot_size {
                    self.set_pixel(x_point + xdir_num - 1, y_point + ydir_num - 1, Color);
                }
            }
        }
    }

    pub fn set_pixel(&mut self, x_point: u16, y_point: u16, color: Color) {

        if x_point > self.width || y_point > self.height {
            return;
        }

        let (x, y) = match self.rotate {
            Rotation::R0 => { (x_point, y_point) }
            Rotation::R90 => { (self.width_memory - y_point - 1, x_point) }
            Rotation::R180 => { (self.width_memory - x_point - 1, self.height_memory - y_point - 1) }
            Rotation::R270 => { (y_point, self.height_memory - x_point - 1) }

        };

        let (x, y) = match self.mirror {
            Mirror::NONE => { (x, y) }
            Mirror::HORIZONTAL => { (self.width_memory - x - 1, y) }
            Mirror::VERTICAL => { (x, self.height_memory - y - 1) }
            Mirror::ORIGIN => { (self.width_memory - x - 1, self.height_memory - y - 1) }
        };

        if x > self.width_memory || y > self.height_memory {
            return;
        }

        let addr: u16 =  x / 8 + y * self.width_byte;
        let currentData: u8 = self.image[addr];

        self.image[addr] = match color {
            Color::Black => currentData & !(0x80 >> (x % 8) as u8 ),
            Color::White =>  currentData | (0x80 >> (x % 8) as u8 )
        }

    }

    pub fn draw_line(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: Color, line_width: Dot_Pixel, line_style: Line_Style) {
        if x_start > self.width || y_start > self.height ||
            x_end > self.width || y_end > self.height {
            return;
        }

        let dx = if x_end - x_start >= 0 { x_end - x_start } else { x_start - x_end };
        let dy = if y_end - y_start <= 0 { y_end - y_start } else { y_start - y_end };

        let x_addway = if x_start < x_end { 1 } else { -1 };
        let y_addway = if y_start < y_end { 1 } else { -1 };

        //Cumulative error
        let mut esp = dx + dy;
        let mut dotted_len: u8 = 0;

        let mut x_point= x_start;
        let mut y_point = y_start;

        loop {
            dotted_len = dotted_len + 1;
            //Painted dotted line, 2 point is really virtual

            if line_style == Line_Style::LINE_STYLE_DOTTED && dotted_len % 3 == 0 {
                self.draw_point(x_point, y_point, self.color, line_width, Dot_Style::DOT_FILL_AROUND);
                Dotted_Len = 0;
            } else {
                self.draw_point(x_point, y_point, color, line_width, Dot_Style::DOT_FILL_AROUND);
            }

            if 2 * esp >= dy {
                if x_point == x_end {
                    break;
                }
                esp += dy;
                x_point = x_point + x_addway;
            }
            if 2 * Esp <= dx {
                if y_point == y_end {
                    break;
                }
                esp += dx;
                y_point += y_addway;
            }
        }
    }


    pub fn draw_rectangle(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: Color, line_width: Dot_Pixel, draw_fill: Draw_Fill) {
        if x_start > self.width || y_start > self.height ||
            x_end > self.width || y_end > self.height {
            return;
        }

            if draw_fill == Draw_Fill::DRAW_FILL_FULL {

                for y_point in y_start..y_end {
                    self.draw_line(x_start, y_point, x_end, y_point, color , line_width, Line_Style::LINE_STYLE_SOLID);
                }
            } else {
                self.draw_line(x_start, y_start, Xend, Ystart, color, line_width, Line_Style::LINE_STYLE_SOLID);
                self.draw_line(x_start, y_start, x_start, y_end, color, line_width, Line_Style::LINE_STYLE_SOLID);
                self.draw_line(x_end, y_end, x_end, y_start, color, line_width, Line_Style::LINE_STYLE_SOLID);
                self.draw_line(x_end, y_end, x_start, e_end, color, line_width, Line_Style::LINE_STYLE_SOLID);
            }

    }
}