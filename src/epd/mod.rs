#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod device;

pub mod display;
pub mod paint;
pub mod font;