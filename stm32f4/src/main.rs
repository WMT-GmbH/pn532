#![no_main]
#![no_std]

mod bootload;
mod logger;
mod println;
mod serial;

use cortex_m_rt::{entry, pre_init};
use panic_persist::get_panic_message_bytes;
use stm32f4xx_hal::i2c::I2c;
use stm32f4xx_hal::otg_fs::USB;
use stm32f4xx_hal::{pac, prelude::*};

use crate::serial::SERIAL;
use pn532::i2c::I2CInterface;
use pn532::{Pn532, Request};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // ---------- Clocks -----------------
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(12.MHz()).sysclk(168.MHz()).freeze();

    // ---------- GPIO -----------------
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    // Onboard LED
    let mut led_red = gpioa.pa13.into_push_pull_output();
    let mut led_yellow = gpioa.pa15.into_push_pull_output();
    let mut led_green = gpioa.pa14.into_push_pull_output();
    led_red.set_low();
    led_yellow.set_low();
    led_green.set_high();

    // ---------- Timer -----------------
    let timer = cp.SYST.counter::<10_000>(&clocks);

    // ---------- I2C -----------------
    let sda = gpiob.pb11.into_alternate_open_drain();
    let scl = gpiob.pb10.into_alternate_open_drain();

    let i2c = I2c::new(dp.I2C2, (scl, sda), 100.kHz(), &clocks);

    // ---------- USB -----------------
    let usb = USB::new(
        (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
        (gpioa.pa11, gpioa.pa12),
        &clocks,
    );
    let mut usb_dev = serial::init(usb);

    // ---------- Logger -----------------
    logger::init_logger(log::LevelFilter::Debug);

    // ---------- PN532 -----------------
    let interface = I2CInterface { i2c };
    let mut pn532: Pn532<_, _, 32> = Pn532::new(interface, timer);

    // ---------- serial connection and panic message -----------------
    critical_section::with(|cs| {
        let mut serial_ref = SERIAL.borrow_ref_mut(cs);
        let serial = serial_ref.as_mut().unwrap();
        for _ in 0..500_000 {
            usb_dev.poll(&mut [serial]);
        }
        // Check if there was a panic message, if so, send to serial
        if let Some(msg) = get_panic_message_bytes() {
            led_red.set_high();

            serial::write(serial, msg).ok();
        }
    });

    // ---------- loop -----------------
    loop {
        let mut buf = [0];

        let read_res = critical_section::with(|cs| {
            let mut serial_ref = SERIAL.borrow_ref_mut(cs);
            let serial = serial_ref.as_mut().unwrap();
            usb_dev.poll(&mut [serial]);

            serial.read(&mut buf)
        });

        if read_res.is_ok() {
            match buf[0] {
                b'f' => {
                    let res = pn532.process(&Request::GET_FIRMWARE_VERSION, 4, 1000.millis());
                    println!("{:?}", res);
                }
                b'b' => bootload::enter(),
                _ => {
                    println!("asdf");
                }
            }
        }
    }
}

#[pre_init]
unsafe fn pre_init() {
    bootload::check();
}
