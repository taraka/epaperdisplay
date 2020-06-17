

fn main() {
    cc::Build::new()
        .file("lib/font12.c")
        .file("lib/font8.c")
        .file("lib/font20.c")
        .file("lib/font24.c")
        .file("lib/font16.c")
        .compile("apd.a")
}