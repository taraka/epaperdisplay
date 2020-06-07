extern {
    fn EPD_7in5_V2_test();
}

fn main() {
    println!("Hello, world!");
    unsafe { EPD_7in5_V2_test() }
}
