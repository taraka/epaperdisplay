use crate::epd::paint::Image;
use crate::epd::device::*;

pub const WIDTH: u16 = 800;
pub const HEIGHT: u16 = 480;

pub struct Display {
    pi: Pi
}

impl Display {
    pub fn init() -> Display {

        let mut this = Display {
            pi: Pi::init()

        };

        this.reset();

        this.send_command(0x01);			//POWER SETTING
        this.send_data(0x07);
        this.send_data(0x07);    //VGH=20V,VGL=-20V
        this.send_data(0x3f);		//VDH=15V
        this.send_data(0x3f);		//VDL=-15V

        this.send_command(0x04); //POWER ON
        this.pi.delay_ms(100);
        this.wait_until_idle();

        this.send_command(0x00);			//PANNEL SETTING
        this.send_data(0x1F);   //KW-3f   KWR-2F	BWROTP 0f	BWOTP 1f

        this.send_command(0x61);        	//tres
        this.send_data(0x03);		//source 800
        this.send_data(0x20);
        this.send_data(0x01);		//gate 480
        this.send_data(0xE0);

        this.send_command(0x15);
        this.send_data(0x00);

        this.send_command(0x50);			//VCOM AND DATA INTERVAL SETTING
        this.send_data(0x10);
        this.send_data(0x07);

        this.send_command(0x60);			//TCON SETTING
        this.send_data(0x22);

        return this;
    }


    pub fn display(&mut self, image: Image) {
        let my_width = WIDTH / 8;

        self.send_command(0x13);
        for j in 0..HEIGHT {
            for i in 0..my_width {
                self.send_data(!image.image[(i + j * my_width) as usize]);
            }
        }
        self.turn_on_display();
    }

    pub fn clear(&mut self) {
        let my_width = WIDTH / 8;

        self.send_command(0x10);
        for _i in 0..(HEIGHT*my_width) {
            self.send_data(0x00);
        }
        self.send_command(0x13);
        for _i in 0..(HEIGHT*my_width)	{
            self.send_data(0x00);
        }
        self.turn_on_display();
    }

    pub fn clear_black(&mut self) {
        let my_width = WIDTH / 8;

        self.send_command(0x13);
        for _i in 0..(HEIGHT*my_width) {
            self.send_data(0xff);
        }

        self.turn_on_display();
    }

    pub fn update_rate() -> u32 {
        15 * 60 * 1000 //15 mins
    }


    fn sleep(&mut self)
    {
        self.send_command(0x02);  	//power off
        self.wait_until_idle();
        self.send_command(0x07);  	//deep sleep
        self.send_data(0xA5);
    }

    fn reset(&mut self)
    {
        self.pi.write(Pin::EPD_RST_PIN, true);
        self.pi.delay_ms(200);
        self.pi.write(Pin::EPD_RST_PIN, false);
        self.pi.delay_ms(2);
        self.pi.write(Pin::EPD_RST_PIN, true);
        self.pi.delay_ms(200);
    }



    fn send_command(&mut self, reg: u8)
    {
        self.pi.write(Pin::EPD_DC_PIN, false);
        self.pi.write(Pin::EPD_CS_PIN, false);
        self.pi.spi_write_byte(reg);
        self.pi.write(Pin::EPD_CS_PIN, true);
    }


    fn send_data(&mut self, data: u8)
    {
        self.pi.write(Pin::EPD_DC_PIN, true);
        self.pi.write(Pin::EPD_CS_PIN, false);
        self.pi.spi_write_byte(data);
        self.pi.write(Pin::EPD_CS_PIN, true);
    }


    fn wait_until_idle(&mut self)
    {
        loop {
            self.send_command(0x71);
            if self.pi.read(Pin::EPD_BUSY_PIN) {
                break;
            }
            self.pi.delay_ms(5);
        }
        self.pi.delay_ms(200);
    }

    fn turn_on_display(&mut self)
    {
        self.send_command(0x12);			//DISPLAY REFRESH
        self.pi.delay_ms(100);	        // The delay here is necessary, 200uS at least!!!
        self.wait_until_idle();
    }
}
