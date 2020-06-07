
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=lib/*.c");

    //This doesn't seem to be working so added the lib manually for now
    println!("cargo:rustc-link-lib=wiringPi");

    // Use the `cc` crate to build a C file and statically link it.
    cc::Build::new()
        .file("lib/DEV_Config.c")
        .file("lib/dev_hardware_SPI.c")
        .file("lib/RPI_sysfs_gpio.c")
        .file("lib/GUI_Paint.c")
        .file("lib/GUI_BMPfile.c")
        .file("lib/font12.c")
        .file("lib/font24CN.c")
        .file("lib/font8.c")
        .file("lib/font20.c")
        .file("lib/font24.c")
        .file("lib/font16.c")
        .file("lib/font12CN.c")

        // This should be found by ld but not sure why it wasn't working
        .object("/usr/lib/libwiringPi.so")

        .define("USE_WIRINGPI_LIB", None)
        .define("RPI", None)
        .compile("epd.a");
}