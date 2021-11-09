use core::convert::Infallible;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

use crate::spi::SPIInterface;
use crate::Pn532;

/// used for doc tests
pub fn get_pn532() -> Pn532<SPIInterface<NoOpSPI, NoOpCS>, NoOpTimer> {
    Pn532::new(
        SPIInterface {
            spi: NoOpSPI,
            cs: NoOpCS,
        },
        NoOpTimer,
    )
}

/// used for doc tests
pub fn get_async_pn532() -> Pn532<SPIInterface<NoOpSPI, NoOpCS>, ()> {
    Pn532::new(
        SPIInterface {
            spi: NoOpSPI,
            cs: NoOpCS,
        },
        (),
    )
}

pub struct NoOpCS;
pub struct NoOpSPI;
pub struct NoOpTimer;

impl OutputPin for NoOpCS {
    type Error = Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Write<u8> for NoOpSPI {
    type Error = Infallible;

    fn write(&mut self, _: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Transfer<u8> for NoOpSPI {
    type Error = Infallible;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        Ok(words)
    }
}

impl CountDown for NoOpTimer {
    type Time = MicroSecond;

    fn start<T>(&mut self, _: T)
    where
        T: Into<Self::Time>,
    {
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        nb::Result::Ok(())
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct MicroSecond(pub u32);

pub trait U32Ext {
    fn ms(self) -> MicroSecond;
}

impl U32Ext for u32 {
    fn ms(self) -> MicroSecond {
        MicroSecond(self.saturating_mul(1_000))
    }
}
