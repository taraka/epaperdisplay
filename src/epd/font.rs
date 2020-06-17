mod font8;
mod font12;
mod font16;
mod font20;
mod font24;

pub use font8::FONT8;
pub use font12::FONT12;
pub use font16::FONT16;
pub use font20::FONT20;
pub use font24::FONT24;


pub struct Font
{
    pub table: &'static [u8],
    pub width: u16,
    pub height: u16
}
