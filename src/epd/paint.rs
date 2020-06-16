extern crate bresenham;

use std::ffi::CString;
use std::os::raw::c_char;
use crate::epd::paint::Dot_Pixel::DOT_PIXEL_1X1;
use crate::epd::paint::Dot_Style::DOT_FILL_AROUND;


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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Font
{
    table: *const u8,
    pub width: u16,
    pub height: u16
}

extern "C" {
    pub static Font24: Font;
    pub static Font20: Font;
    pub static Font16: Font;
    pub static Font12: Font;
    pub static Font8: Font;
}

pub type ImageData = Box<[u8]>;

#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    White = 0xff,
    Black = 0x00
}

#[derive(Clone, Copy, PartialEq)]
enum Mirror {
    NONE  = 0,
    HORIZONTAL = 1,
    VERTICAL = 2,
    ORIGIN = 3,
}

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
pub enum Rotation {
    R0  = 0,
    R90 = 90,
    R180 = 180,
    R270 = 270
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dot_Style {
    DOT_FILL_AROUND  = 1,		// dot pixel 1 x 1
    DOT_FILL_RIGHTUP  , 		// dot pixel 2 X 2
}

#[derive(Clone, Copy, PartialEq)]
pub enum Line_Style {
    LINE_STYLE_SOLID = 0,
    LINE_STYLE_DOTTED,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Draw_Fill {
    DRAW_FILL_EMPTY = 0,
    DRAW_FILL_FULL,
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
                for x in 0..self.width_byte {//8 pixel =  1 byte
                    self.image[( x + y * self.width_byte) as usize] = color as u8;
                }
        }
    }

    pub fn draw_point(&mut self, x_point: u16, y_point: u16, color: Color, dot_pixel: Dot_Pixel, dot_style: Dot_Style) {
        if x_point > self.width || y_point > self.height {
            return;
        }

        let dot_size = dot_pixel as u16;

        if dot_style == Dot_Style::DOT_FILL_AROUND {
            for xdir_num in 0..2 * dot_pixel as u16 - 1 {
                for ydir_num in 0..2 * dot_pixel as u16 - 1 {
                    if (x_point as i32 + xdir_num as i32 - dot_size as i32) < 0 || (y_point as i32 + ydir_num as i32 - dot_size as i32) < 0 {
                        break;
                    }
                    // printf("x = %d, y = %d\r\n", Xpoint + XDir_Num - Dot_Pixel, Ypoint + YDir_Num - Dot_Pixel);
                    self.set_pixel(x_point + xdir_num - dot_size, y_point + ydir_num - dot_size, color);
                }
            }
        } else {
            for xdir_num in  0..dot_size {
                for ydir_num in 0..dot_size {
                    self.set_pixel(x_point + xdir_num - 1, y_point + ydir_num - 1, color);
                }
            }
        }
    }

    pub fn set_pixel(&mut self, x_point: u16, y_point: u16, color: Color) {
        if x_point >= self.width || y_point >= self.height {
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

        let addr =  (x / 8 + y * self.width_byte) as usize;
        let current_data: u8 = self.image[addr];

        self.image[addr] = match color {
            Color::Black => current_data & !(0x80 >> (x % 8) as u8 ),
            Color::White =>  current_data | (0x80 >> (x % 8) as u8 )
        }

    }

    pub fn draw_line(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: Color, line_width: Dot_Pixel, line_style: Line_Style) {
        if x_start > self.width || y_start > self.height ||
            x_end > self.width || y_end > self.height {
            return;
        }

        let mut dotted_len: u16 = 0;

        for (x, y) in bresenham::Bresenham::new((x_start as isize, y_start as isize), (x_end as isize, y_end as isize)) {
            dotted_len += 1;
            if line_style == Line_Style::LINE_STYLE_DOTTED && dotted_len % 3 == 0 {
                self.draw_point(x as u16, y as u16, self.color, line_width, Dot_Style::DOT_FILL_AROUND);
                dotted_len = 0;
            } else {
                self.draw_point(x as u16, y as u16, color, line_width, Dot_Style::DOT_FILL_AROUND);
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
            self.draw_line(x_start, y_start, x_end, y_start, color, line_width, Line_Style::LINE_STYLE_SOLID);
            self.draw_line(x_start, y_start, x_start, y_end, color, line_width, Line_Style::LINE_STYLE_SOLID);
            self.draw_line(x_end, y_end, x_end, y_start, color, line_width, Line_Style::LINE_STYLE_SOLID);
            self.draw_line(x_start, y_end, x_end , y_end, color, line_width, Line_Style::LINE_STYLE_SOLID);
        }
    }

    pub fn draw_circle(&mut self, x_center: u16, y_center: u16, radius: u16, color: Color, line_width: Dot_Pixel, draw_fill: Draw_Fill) {
        if x_center > self.width || y_center >= self.height {
            return;
        }

        let mut x  = 0;
        let mut y = radius;

        //Cumulative error,judge the next point of the logo
        let mut esp = 3 - (radius << 1 ) as i32;

        if draw_fill == Draw_Fill::DRAW_FILL_FULL {
            while x <= y { //Realistic circles
                for cy in x..y+1 {
                    self.draw_point(x_center + x, y_center + cy, color, DOT_PIXEL_1X1, DOT_FILL_AROUND);//1
                    self.draw_point(x_center - x, y_center + cy, color, DOT_PIXEL_1X1, DOT_FILL_AROUND);//2
                    self.draw_point(x_center - cy, y_center + x, color, DOT_PIXEL_1X1, DOT_FILL_AROUND);//3
                    self.draw_point(x_center - cy, y_center - x, color, DOT_PIXEL_1X1, DOT_FILL_AROUND);//4
                    self.draw_point(x_center - x, y_center - cy, color, DOT_PIXEL_1X1, DOT_FILL_AROUND);//5
                    self.draw_point(x_center + x, y_center - cy, color, DOT_PIXEL_1X1, DOT_FILL_AROUND);//6
                    self.draw_point(x_center + cy, y_center - x, color, DOT_PIXEL_1X1, DOT_FILL_AROUND);//7
                    self.draw_point(x_center + cy, y_center + x, color, DOT_PIXEL_1X1, DOT_FILL_AROUND);
                }
                if esp < 0 {
                    esp += 4 * x as i32 + 6;
                }
                else {
                    esp += 10 + 4 * (x as i32 - y as i32);
                    y -= 1;
                }
                x += 1;
            }
        } else { //Draw a hollow circle
            while x <= y {
                self.draw_point(x_center + x, y_center + y, color, line_width, DOT_FILL_AROUND);//1
                self.draw_point(x_center - x, y_center + y, color, line_width, DOT_FILL_AROUND);//2
                self.draw_point(x_center - y, y_center + x, color, line_width, DOT_FILL_AROUND);//3
                self.draw_point(x_center - y, y_center - x, color, line_width, DOT_FILL_AROUND);//4
                self.draw_point(x_center - x, y_center - y, color, line_width, DOT_FILL_AROUND);//5
                self.draw_point(x_center + x, y_center - y, color, line_width, DOT_FILL_AROUND);//6
                self.draw_point(x_center + y, y_center - x, color, line_width, DOT_FILL_AROUND);//7
                self.draw_point(x_center + y, y_center + x, color, line_width, DOT_FILL_AROUND);//0

                if esp < 0 {
                    esp += 4 * x as i32 + 6;
                }
                else {
                    esp += 10 + 4 * (x as i32 - y as i32);
                    y -= 1;
                }
                x += 1;
            }
        }
    }

    pub fn draw_string(&mut self, x_start: u16, y_start: u16, string: &str, font: Box<Font>, fg_color: Color, bg_color: Color) -> (u16, u16){
        if x_start > self.width || y_start + font.height > self.height {
            return (x_start, y_start);
        }

        let mut x = x_start;
        let mut y = y_start;
        let mut max_x = x;

        for (_, c) in string.chars().enumerate() {
            //if X direction filled , reposition to(Xstart,Ypoint),Ypoint is Y direction plus the Height of the character
            if (x + font.width ) > self.width {
                x = x_start;
                y += font.height;
            }

            // If the Y direction is full, reposition to(Xstart, Ystart)
            if (y  + font.height ) > self.height {
                x = x_start;
                y = y_start;
            }
            self.draw_char(x, y, c, &font, fg_color, bg_color);

            x += font.width;
            if x > max_x {
                max_x = x;
            }
        }

        (max_x + font.width, y + font.height)
    }

    pub fn draw_char(&mut self, x_start: u16, y_start: u16, c: char, font: &Box<Font>, fg_color: Color, bg_color: Color) {
        if x_start > self.width || y_start > self.height {
            return;
        }

        let mut offset = (c as u16 - ' ' as u16) * font.height * (font.width / 8 + (if font.width % 8 != 0 { 1 } else { 0 }));

        for page in 0..font.height {
            for column in 0..font.width {

                let data = unsafe { *font.table.offset(offset as isize) };

                //To determine whether the font background color and screen background color is consistent
                if bg_color == self.color { //this process is to speed up the scan
                    if data as u16 & (0x80 >> (column % 8)) != 0 {
                        self.set_pixel(x_start + column, y_start + page, fg_color);
                    }
                    // Paint_DrawPoint(Xpoint + Column, Ypoint + Page, Color_Foreground, DOT_PIXEL_DFT, DOT_STYLE_DFT);
                } else {
                    if data as u16 & (0x80 >> (column % 8)) != 0 {
                        self.set_pixel(x_start + column, y_start + page, fg_color);
                        // Paint_DrawPoint(Xpoint + Column, Ypoint + Page, Color_Foreground, DOT_PIXEL_DFT, DOT_STYLE_DFT);
                    } else {
                        self.set_pixel(x_start + column, y_start + page, bg_color);
                        // Paint_DrawPoint(Xpoint + Column, Ypoint + Page, Color_Background, DOT_PIXEL_DFT, DOT_STYLE_DFT);
                    }
                }
                //One pixel is 8 bits
                if column % 8 == 7 {
                    offset += 1;
                }
            }// Write a line
            if font.width % 8 != 0 {
                offset += 1;
            }
        }// Write all
    }

    pub fn draw_num(&mut self, x_start: u16, y_start: u16, number: i32, font: Box<Font>, fg_color: Color, bg_color: Color) {
        self.draw_string(x_start, y_start, &format!("{}", number)[..], font, fg_color, bg_color);
    }

}