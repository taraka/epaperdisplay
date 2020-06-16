
#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), path = "vdevice.rs")]
mod device;

pub mod display;
pub mod paint;