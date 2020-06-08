
#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), path = "vdevice.rs")]
pub mod device;


pub mod display;
pub mod paint;
pub mod bmp;
pub mod bresenham;