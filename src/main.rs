//! This example is for the STM32F103 "Blue Pill" board using I2C1.
//!
//! Wiring connections are as follows for a CRIUS-branded display:
//!
//! ```
//!      Display -> Blue Pill
//! (black)  GND -> GND
//! (red)    +5V -> VCC
//! (yellow) SDA -> PB7
//! (green)  SCL -> PB6
//! ```
//!

#![no_std]
#![no_main]

use cortex_m_rt::{entry};
use panic_halt as _;
use rand::prelude::*;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f1xx_hal::{
    i2c::{BlockingI2c, DutyCycle, Mode},
    prelude::*,
    stm32,
    adc,
};


   
const W: usize = 128;
const H: usize = 64;
const BUFF_SIZE: usize = W*H/8;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();


    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();


    let clocks = rcc.cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(6.MHz())
        .freeze(&mut flash.acr);


    let mut afio = dp.AFIO.constrain();

    let mut gpiob = dp.GPIOB.split();

    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);


    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400.kHz(),
            duty_cycle: DutyCycle::Ratio16to9,
        },
        clocks,
        1000,
        10,
        1000,
        1000,
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0);
    display.init().unwrap();
    display.clear().unwrap();


    fn get_xy(x: usize, y: usize, buf: [u8 ; BUFF_SIZE]) -> u8 {
        if (buf[x + ((y / 8) * W)] & 0b00000001 << (y % 8)) != 0 {
            return 1
        }
        return 0
    }

    fn toggle_xy(x: usize, y: usize,  buf: &mut [u8 ; BUFF_SIZE]) {
        buf[x + ((y / 8) * W)] ^= 0b00000001 << (y % 8);
    }

    let mut buf = [0x00u8; BUFF_SIZE];

    let mut delay = cp.SYST.delay(&clocks);

    let mut ch0 = gpiob.pb0.into_analog(&mut gpiob.crl);
    let mut adc1 = adc::Adc::adc1(dp.ADC1, clocks);

    let mut rng = SmallRng::seed_from_u64(adc1.read(&mut ch0).unwrap());
    rng.fill_bytes(&mut buf);

    let mut swap = [0x00u8; BUFF_SIZE];
    
    for x in 0..BUFF_SIZE {
        swap[x] = buf[x];
    }

    display.draw(&buf).unwrap();

    loop {
        for y in 0..H {
            for x in 0..W {
                
                let mut live_neighbours: u8 = 0;
                
                let x_avoid_underfow = if x == 0 {W-1} else {x};
                let y_avoid_underfow = if y == 0 {H-1} else {y};

                let x_avoid_overflow = if x == W-1 {0} else {x};
                let y_avoid_overflow = if y == H-1 {0} else {y};

                live_neighbours += get_xy(x_avoid_underfow-1, y, buf);
                live_neighbours += get_xy(x, y_avoid_underfow-1, buf);
                
                live_neighbours += get_xy(x_avoid_overflow+1, y, buf);
                live_neighbours += get_xy(x, y_avoid_overflow+1, buf);
                
                live_neighbours += get_xy(x_avoid_underfow-1, y_avoid_underfow-1, buf);
                live_neighbours += get_xy(x_avoid_overflow+1, y_avoid_overflow+1, buf);

                live_neighbours += get_xy(x_avoid_underfow-1, y_avoid_overflow+1, buf);
                live_neighbours += get_xy(x_avoid_overflow+1, y_avoid_underfow-1, buf);


                // it's a live cell
                if get_xy(x, y, buf) == 1 {
                    // Any live cell with fewer than two live neighbours dies, as if caused by underpopulation.
                    // Any live cell with more than three live neighbours dies, as if by overpopulation.
                    if live_neighbours < 2 || live_neighbours > 3 {
                        toggle_xy(x, y, &mut swap);
                    }
                } else {
                    // Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
                    if live_neighbours == 3 {
                        toggle_xy(x, y, &mut swap);
                    }
                }
            }
        }

        for x in 0..BUFF_SIZE {
            buf[x] = swap[x];
        }

        display.draw(&buf).unwrap();
        delay.delay_ms(10u16);
    }
}

