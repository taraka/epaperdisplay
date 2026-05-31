#[cfg(feature = "virtual")]
mod vdisplay;

#[cfg(all(not(feature = "virtual"), not(any(target_os = "macos", target_os = "windows"))))]
mod d7in5_v2;

#[cfg(all(not(feature = "virtual"), any(target_os = "macos", target_os = "windows")))]
compile_error!("Use `--features virtual` when building on macOS or Windows.");

#[cfg(feature = "virtual")]
pub use vdisplay::{Display, HEIGHT, UPDATE_RATE, WIDTH};

#[cfg(all(not(feature = "virtual"), not(any(target_os = "macos", target_os = "windows"))))]
pub use d7in5_v2::{Display, HEIGHT, UPDATE_RATE, WIDTH};
