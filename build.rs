
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=lib/*.c");
    println!("cargo:rustc-link-lib=wiringPi");
    // Use the `cc` crate to build a C file and statically link it.
    cc::Build::new()
        //.file("lib/EPD_7in5_V2.h")
        //.file("lib/GUI_BMPfile.h")
        //.file("lib/fonts.h")
        .file("lib/font12.c")
        .file("lib/font24CN.c")
        //.file("lib/DEV_Config.h")
        //.file("lib/GUI_Paint.h")
        .file("lib/font8.c")
        .file("lib/font20.c")
        .file("lib/font24.c")
        .file("lib/font16.c")
        .file("lib/DEV_Config.c")
        .file("lib/lib.c")
        .file("lib/font12CN.c")
        .file("lib/GUI_Paint.c")
        .file("lib/GUI_BMPfile.c")
        .file("lib/EPD_7in5_V2.c")
            .compile("epd.a");
}