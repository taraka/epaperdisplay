use crate::epd::paint::Image;
use crate::epd::device::*;


extern {
    fn EPD_7IN5_V2_Clear();
    fn EPD_7IN5_V2_Display(image: *mut u8);
    fn EPD_7IN5_V2_Sleep();
}

pub const WIDTH: u16 = 800;
pub const HEIGHT: u16 = 480;

pub fn clear() {
    let my_width = WIDTH / 8;

    send_command(0x10);
    for i in 0..(HEIGHT*my_width) {
        send_data(0x00);
    }
    send_command(0x13);
    for i in 0..(HEIGHT*my_width)	{
        send_data(0x00);
    }
    turn_on_display();
}

pub fn display(image: &mut Image) {
    unsafe { EPD_7IN5_V2_Display(image.as_mut_ptr()) }
}

pub fn sleep() {
    unsafe { EPD_7IN5_V2_Sleep(); }
}



pub fn init() {

    reset();

    send_command(0x01);			//POWER SETTING
    send_data(0x07);
    send_data(0x07);    //VGH=20V,VGL=-20V
    send_data(0x3f);		//VDH=15V
    send_data(0x3f);		//VDL=-15V

    send_command(0x04); //POWER ON
    delay_ms(100);
    wait_until_idle();

    send_command(0x00);			//PANNEL SETTING
    send_data(0x1F);   //KW-3f   KWR-2F	BWROTP 0f	BWOTP 1f

    send_command(0x61);        	//tres
    send_data(0x03);		//source 800
    send_data(0x20);
    send_data(0x01);		//gate 480
    send_data(0xE0);

    send_command(0x15);
    send_data(0x00);

    send_command(0x50);			//VCOM AND DATA INTERVAL SETTING
    send_data(0x10);
    send_data(0x07);

    send_command(0x60);			//TCON SETTING
    send_data(0x22);
}


pub fn reset()
{
    digital_write(Pin::EPD_RST_PIN, 1);
    delay_ms(200);
    digital_write(Pin::EPD_RST_PIN, 0);
    delay_ms(2);
    digital_write(Pin::EPD_RST_PIN, 1);
    delay_ms(200);
}



fn send_command(reg: u8)
{
    digital_write(Pin::EPD_DC_PIN, 0);
    digital_write(Pin::EPD_CS_PIN, 0);
    spi_write_byte(reg);
    digital_write(Pin::EPD_CS_PIN, 1);
}


fn send_data(data: u8)
{
    digital_write(Pin::EPD_DC_PIN, 1);
    digital_write(Pin::EPD_CS_PIN, 0);
    spi_write_byte(data);
    digital_write(Pin::EPD_CS_PIN, 1);
}


fn wait_until_idle()
{
    loop {
        send_command(0x71);
        if (digital_read(Pin::EPD_BUSY_PIN) & 0x01) == 0x01 {
            break;
        }
        println!("Waiting");
        delay_ms(5);
    }
    delay_ms(200);
}

fn turn_on_display()
{
    send_command(0x12);			//DISPLAY REFRESH
    delay_ms(100);	        // The delay here is necessary, 200uS at least!!!
    wait_until_idle();
}