use ::bresenham;

use crate::epd::paint::DotPixel::*;
use crate::epd::paint::DotStyle::*;
use crate::epd::font::*;

#[derive(PartialEq)]
pub struct Image {
    pub(crate) image: ImageData,
    // Second bitplane for the red/black/white panel. Same bit convention as
    // `image` (1 = inactive here, 0 = ink here) but sent to a separate
    // display register. Unused — and always blank — on the plain b/w panel.
    pub(crate) red: ImageData,
    width: u16,
    height: u16,
    width_memory: u16,
    height_memory: u16,
    color: Color,
    rotate: Rotation,
    mirror: Mirror,
    width_byte: u16,
    height_byte: u16
}

pub type ImageData = Box<[u8]>;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
    Red,
}
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
enum Mirror {
    None = 0,
    Horizontal = 1,
    Vertical = 2,
    Origin = 3,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum DotPixel {
    DotPixel1x1 = 1,
    DotPixel2x2,
    DotPixel3x3,
    DotPixel4x4,
    DotPixel5x5,
    DotPixel6x6,
    DotPixel7x7,
    DotPixel8x8,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum Rotation {
    R0  = 0,
    R90 = 90,
    R180 = 180,
    R270 = 270
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum DotStyle {
    DotFillAround = 1,		// dot pixel 1 x 1
    DotFillRightup, 		// dot pixel 2 X 2
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum LineStyle {
    LineStyleSolid = 0,
    LineStyleDotted,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum DrawFill {
    DrawFillEmpty = 0,
    DrawFillFull,
}



pub fn new_image(width: u16, height: u16, color: Color) -> Image {

    let image_size: usize = ( if width % 8 == 0 { width / 8 } else { width / 8 + 1} ) as usize * height as usize;

    Image {
        image: vec![0; image_size].into_boxed_slice(),
        red: vec![0; image_size].into_boxed_slice(),
        width_memory: width,
        height_memory: height,
        color,
        width_byte: width / 8,
        height_byte: height,
        rotate: Rotation::R0,
        mirror: Mirror::None,
        width,
        height
    }
}

impl Image {

    #[allow(dead_code)]
    pub fn clear(&mut self, color: Color) {
        let (black_fill, red_fill): (u8, u8) = match color {
            Color::White => (0xff, 0xff),
            Color::Black => (0x00, 0xff),
            Color::Red   => (0xff, 0x00),
        };
        for y in  0..self.height_byte {
            for x in 0..self.width_byte {//8 pixel =  1 byte
                let addr = (x + y * self.width_byte) as usize;
                self.image[addr] = black_fill;
                self.red[addr] = red_fill;
            }
        }
    }

    #[allow(dead_code)]
    pub fn draw_point(&mut self, x_point: u16, y_point: u16, color: Color, dot_pixel: DotPixel, dot_style: DotStyle) {
        if x_point > self.width || y_point > self.height {
            return;
        }

        let dot_size = dot_pixel as u16;

        if dot_style == DotStyle::DotFillAround {
            for xdir_num in 0..2 * dot_pixel as u16 - 1 {
                for ydir_num in 0..2 * dot_pixel as u16 - 1 {
                    if (x_point as i32 + xdir_num as i32 - dot_size as i32) < 0 || (y_point as i32 + ydir_num as i32 - dot_size as i32) < 0 {
                        break;
                    }
                    // printf("x = %d, y = %d\r\n", Xpoint + XDir_Num - DotPixel, Ypoint + YDir_Num - DotPixel);
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

    #[allow(dead_code)]
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
            Mirror::None => { (x, y) }
            Mirror::Horizontal => { (self.width_memory - x - 1, y) }
            Mirror::Vertical => { (x, self.height_memory - y - 1) }
            Mirror::Origin => { (self.width_memory - x - 1, self.height_memory - y - 1) }
        };

        if x > self.width_memory || y > self.height_memory {
            return;
        }

        let addr =  (x / 8 + y * self.width_byte) as usize;
        let mask = 0x80 >> (x % 8) as u8;

        // Each color inks exactly one plane and clears the other, so a pixel
        // never shows ink on both at once.
        let (black_bit, red_bit) = match color {
            Color::White => (true, true),
            Color::Black => (false, true),
            Color::Red   => (true, false),
        };

        self.image[addr] = if black_bit { self.image[addr] | mask } else { self.image[addr] & !mask };
        self.red[addr] = if red_bit { self.red[addr] | mask } else { self.red[addr] & !mask };
    }

    #[allow(dead_code)]
    pub fn draw_line(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: Color, line_width: DotPixel, line_style: LineStyle) {
        if x_start > self.width || y_start > self.height ||
            x_end > self.width || y_end > self.height {
            return;
        }

        let mut dotted_len: u16 = 0;

        for (x, y) in bresenham::Bresenham::new((x_start as isize, y_start as isize), (x_end as isize, y_end as isize)) {
            dotted_len += 1;
            if line_style == LineStyle::LineStyleDotted && dotted_len % 3 == 0 {
                self.draw_point(x as u16, y as u16, self.color, line_width, DotStyle::DotFillAround);
                dotted_len = 0;
            } else {
                self.draw_point(x as u16, y as u16, color, line_width, DotStyle::DotFillAround);
            }
        }
    }

    #[allow(dead_code)]
    pub fn draw_rectangle(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: Color, line_width: DotPixel, draw_fill: DrawFill) {
        if x_start > self.width || y_start > self.height ||
            x_end > self.width || y_end > self.height {
            return;
        }

        if draw_fill == DrawFill::DrawFillFull {

            for y_point in y_start..y_end {
                self.draw_line(x_start, y_point, x_end, y_point, color, line_width, LineStyle::LineStyleSolid);
            }
        } else {
            self.draw_line(x_start, y_start, x_end, y_start, color, line_width, LineStyle::LineStyleSolid);
            self.draw_line(x_start, y_start, x_start, y_end, color, line_width, LineStyle::LineStyleSolid);
            self.draw_line(x_end, y_end, x_end, y_start, color, line_width, LineStyle::LineStyleSolid);
            self.draw_line(x_start, y_end, x_end, y_end, color, line_width, LineStyle::LineStyleSolid);
        }
    }

    #[allow(dead_code)]
    pub fn draw_circle(&mut self, x_center: u16, y_center: u16, radius: u16, color: Color, line_width: DotPixel, draw_fill: DrawFill) {
        if x_center > self.width || y_center >= self.height {
            return;
        }

        let mut x  = 0;
        let mut y = radius;

        //Cumulative error,judge the next point of the logo
        let mut esp = 3 - (radius << 1 ) as i32;

        if draw_fill == DrawFill::DrawFillFull {
            while x <= y { //Realistic circles
                for cy in x..y+1 {
                    self.draw_point(x_center + x, y_center + cy, color, DotPixel1x1, DotFillAround);//1
                    self.draw_point(x_center - x, y_center + cy, color, DotPixel1x1, DotFillAround);//2
                    self.draw_point(x_center - cy, y_center + x, color, DotPixel1x1, DotFillAround);//3
                    self.draw_point(x_center - cy, y_center - x, color, DotPixel1x1, DotFillAround);//4
                    self.draw_point(x_center - x, y_center - cy, color, DotPixel1x1, DotFillAround);//5
                    self.draw_point(x_center + x, y_center - cy, color, DotPixel1x1, DotFillAround);//6
                    self.draw_point(x_center + cy, y_center - x, color, DotPixel1x1, DotFillAround);//7
                    self.draw_point(x_center + cy, y_center + x, color, DotPixel1x1, DotFillAround);
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
                self.draw_point(x_center + x, y_center + y, color, line_width, DotFillAround);//1
                self.draw_point(x_center - x, y_center + y, color, line_width, DotFillAround);//2
                self.draw_point(x_center - y, y_center + x, color, line_width, DotFillAround);//3
                self.draw_point(x_center - y, y_center - x, color, line_width, DotFillAround);//4
                self.draw_point(x_center - x, y_center - y, color, line_width, DotFillAround);//5
                self.draw_point(x_center + x, y_center - y, color, line_width, DotFillAround);//6
                self.draw_point(x_center + y, y_center - x, color, line_width, DotFillAround);//7
                self.draw_point(x_center + y, y_center + x, color, line_width, DotFillAround);//0

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

    #[allow(dead_code)]
    pub fn draw_string(&mut self, x_start: u16, y_start: u16, string: &str, font: &Font, fg_color: Color, bg_color: Color) -> (u16, u16){
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


    pub fn draw_char(&mut self, x_start: u16, y_start: u16, ci: char, font: &Font, fg_color: Color, bg_color: Color) {
        let c = if ci as u8 == 25 {
            '\''
        }
        else {
            ci
        };

        if x_start > self.width || y_start > self.height {
            return;
        }

        let mut offset = (c as u16 - ' ' as u16) * font.height * (font.width / 8 + (if font.width % 8 != 0 { 1 } else { 0 }));

        for page in 0..font.height {
            for column in 0..font.width {

                let data = match font.table.get(offset as usize) {
                    Some(d) => *d,
                    _ => 31,
                };

                //To determine whether the font background color and screen background color is consistent
                if bg_color == self.color { //this process is to speed up the scan
                    if data & (0x80 >> (column % 8)) != 0 {
                        self.set_pixel(x_start + column, y_start + page, fg_color);
                    }
                    // Paint_DrawPoint(Xpoint + Column, Ypoint + Page, Color_Foreground, DOT_PIXEL_DFT, DOT_STYLE_DFT);
                } else {
                    if data & (0x80 >> (column % 8)) != 0 {
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

    #[allow(dead_code)]
    pub fn draw_num(&mut self, x_start: u16, y_start: u16, number: i32, font: &Font, fg_color: Color, bg_color: Color) {
        self.draw_string(x_start, y_start, &format!("{}", number)[..], font, fg_color, bg_color);
    }

    // Encodes both bitplanes as a plain uncompressed 24-bit BMP, for serving
    // over HTTP — browsers read BMP natively, so no PNG/image-crate dependency
    // is needed.
    pub fn to_bmp(&self) -> Vec<u8> {
        let row_size = ((self.width as u32 * 3 + 3) / 4) * 4; // rows padded to 4 bytes
        let pixel_data_size = row_size * self.height as u32;
        let header_size = 14 + 40;
        let file_size = header_size + pixel_data_size;

        let mut buf = Vec::with_capacity(file_size as usize);

        // BITMAPFILEHEADER
        buf.extend_from_slice(b"BM");
        buf.extend_from_slice(&file_size.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes()); // reserved
        buf.extend_from_slice(&(header_size as u32).to_le_bytes()); // pixel data offset

        // BITMAPINFOHEADER
        buf.extend_from_slice(&40u32.to_le_bytes()); // header size
        buf.extend_from_slice(&(self.width as i32).to_le_bytes());
        buf.extend_from_slice(&(self.height as i32).to_le_bytes()); // positive = bottom-up
        buf.extend_from_slice(&1u16.to_le_bytes()); // color planes
        buf.extend_from_slice(&24u16.to_le_bytes()); // bits per pixel
        buf.extend_from_slice(&0u32.to_le_bytes()); // compression: BI_RGB (none)
        buf.extend_from_slice(&pixel_data_size.to_le_bytes());
        buf.extend_from_slice(&2835i32.to_le_bytes()); // ~72 DPI
        buf.extend_from_slice(&2835i32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes()); // colors in palette
        buf.extend_from_slice(&0u32.to_le_bytes()); // important colors

        for y in (0..self.height).rev() { // BMP rows are bottom-up
            let mut written = 0u32;
            for x in 0..self.width {
                let addr = (x / 8 + y * self.width_byte) as usize;
                let mask = 0x80 >> (x % 8) as u8;
                let black = self.image[addr] & mask == 0;
                let red = self.red[addr] & mask == 0;

                let (b, g, r) = if red {
                    (40u8, 40u8, 200u8)
                } else if black {
                    (0, 0, 0)
                } else {
                    (255, 255, 255)
                };
                buf.push(b);
                buf.push(g);
                buf.push(r);
                written += 3;
            }
            while written % 4 != 0 {
                buf.push(0);
                written += 1;
            }
        }

        buf
    }

}

