#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
mod device;

pub mod display;
pub mod paint;