extern {
    fn Paint_NewImage(image: *mut u8, width: u16, height: u16, rotate: u16, color: u16);
    fn Paint_SelectImage(image: *mut u8);
    fn Paint_Clear(color: u16);
    fn Paint_DrawBitMap(image: *const u8);
}

pub type Image = Box<[u8]>;

pub enum Color {
    White = 0xff,
    Black = 0x00
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