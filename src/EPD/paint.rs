extern {
    fn Paint_NewImage(image: *u8, width: u16, height: u16, rotate: u16, color: u16);
}

pub type Image = Box<[u8]>;

pub enum Color {
    White = 0xff,
    Black = 0x00
}

pub fn new_image(width: u16, height: u16) ->  {
    let image_size: usize = ( if width % 8 == 0 { width / 8 } else { width / 8 + 1} ) as usize * height as usize;
    let mut image : Box<[u8]> = vec![0; image_size].into_boxed_slice();

    unsafe { Paint_NewImage(image.as_mut_ptr(), width, height, 0, Color::White as u16) };

    return image;
}