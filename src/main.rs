mod EPD;

extern {
    fn EPD_7in5_V2_test();
}

fn main() {
    println!("EPD_7IN5_V2_test Demo");

    EPD::device::module_init().expect("Fail to init device with code: {}");

    println!("e-Paper Init and Clear...");
    EPD::display::d7in5_v2::init();
    EPD::display::d7in5_v2::clear();
    EPD::device::delay_ms(500);


    println!("Paint_NewImage");

    let black_image = EPD::paint::new_image(EPD::display::d7in5_v2::WIDTH, EPD::display::d7in5_v2::HEIGHT);

}


