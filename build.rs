

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=lib/*.c");

    // Use the `cc` crate to build a C file and statically link it.
    cc::Build::new()
        .file("lib/font12.c")
        .file("lib/font8.c")
        .file("lib/font20.c")
        .file("lib/font24.c")
        .file("lib/font16.c")
        .compile("epd.a");
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn main() {
    cc::Build::new()
        .file("lib/font12.c")
        .file("lib/font8.c")
        .file("lib/font20.c")
        .file("lib/font24.c")
        .file("lib/font16.c")
        .compile("apd.a")
}