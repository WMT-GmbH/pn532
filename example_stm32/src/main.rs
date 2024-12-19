#![no_main]
#![no_std]

extern crate panic_rtt_target;

use core::convert::Infallible;
use stm32f4xx_hal as hal;

use cortex_m_rt::entry;
use embedded_hal::spi::MODE_0;
use fugit::TimerDurationU32;
use hal::{pac, prelude::*};
use pn532::spi::SPIInterface;
use pn532::{nb, CountDown, Interface, Pn532, Request};
use rtt_target::rprintln;
use stm32f4xx_hal::spi::BitFormat;
use stm32f4xx_hal::timer::Counter;

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();
    let dp = pac::Peripherals::take().expect("cannot take peripherals");

    let clocks = dp.RCC.constrain().cfgr.sysclk(24.MHz()).freeze();

    let gpioa = dp.GPIOA.split();

    let mut spi = dp.SPI1.spi(
        (gpioa.pa5, gpioa.pa6, gpioa.pa7),
        MODE_0,
        3000.kHz(),
        &clocks,
    );
    spi.bit_format(BitFormat::LsbFirst);
    let cs = gpioa.pa4.into_push_pull_output();

    let timer = TimerWrapper {
        timer: dp.TIM2.counter_ms(&clocks),
    };

    let spi = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let interface = SPIInterface { spi };

    let mut pn532: Pn532<_, _, 32> = Pn532::new(interface, timer);

    pn532.interface.write(&mut []).unwrap();

    rprintln!(
        "{:?}",
        pn532.process(&Request::GET_FIRMWARE_VERSION, 4, 100u32.millis())
    );

    rprintln!(
        "{:?}",
        pn532.process(&Request::GET_FIRMWARE_VERSION, 4, 10u32.millis())
    );

    loop {}
}

struct TimerWrapper<T, const FREQ: u32> {
    timer: Counter<T, FREQ>,
}

impl<TIM, const FREQ: u32> CountDown for TimerWrapper<TIM, FREQ>
where
    TIM: stm32f4xx_hal::timer::Instance,
{
    type Time = TimerDurationU32<FREQ>;
    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Self::Time>,
    {
        self.timer.start(timeout.into()).unwrap();
    }

    fn wait(&mut self) -> nb::Result<(), Infallible> {
        match self.timer.wait() {
            Ok(_) => Ok(()),
            Err(nb::Error::WouldBlock) => Err(nb::Error::WouldBlock),
            Err(nb::Error::Other(_)) => unreachable!(),
        }
    }
}
