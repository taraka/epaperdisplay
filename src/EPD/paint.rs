extern {
    fn Paint_NewImage(image: *mut u8, width: u16, height: u16, rotate: u16, color: u16);
    fn Paint_SelectImage(image: *mut u8);
    fn Paint_Clear(color: u16);
    fn Paint_DrawBitMap(image: *const u8);


    //void Paint_DrawPoint(UWORD Xpoint, UWORD Ypoint, UWORD Color, DOT_PIXEL Dot_Pixel, DOT_STYLE Dot_FillWay);
    fn Paint_DrawPoint(x_point: u16, y_point: u16, color: u16, dot_pixel: Dot_Pixel, dot_style: Dot_Style);
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