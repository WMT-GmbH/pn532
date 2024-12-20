use core::convert::Infallible;
use core::time::Duration;

use crate::CountDown;
use embedded_hal::spi::{Operation, SpiDevice};

use crate::spi::SPIInterface;
use crate::Pn532;

/// used for doc tests
pub fn get_pn532() -> Pn532<SPIInterface<NoOpSPI>, NoOpTimer> {
    Pn532::new(SPIInterface { spi: NoOpSPI }, NoOpTimer)
}

/// used for doc tests
pub fn get_async_pn532() -> Pn532<SPIInterface<NoOpSPI>, ()> {
    Pn532::new(SPIInterface { spi: NoOpSPI }, ())
}

pub struct NoOpSPI;
pub struct NoOpTimer;

impl SpiDevice for NoOpSPI {
    fn transaction(&mut self, _operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl embedded_hal::spi::ErrorType for NoOpSPI {
    type Error = Infallible;
}

impl CountDown for NoOpTimer {
    type Time = Duration;

    fn start<T>(&mut self, _: T)
    where
        T: Into<Self::Time>,
    {
    }

    fn wait(&mut self) -> nb::Result<(), Infallible> {
        Ok(())
    }
}

/// used in CountDown example implementation
pub mod esp_hal {
    pub mod timer {
        use super::super::*;

        #[allow(unused)]
        pub struct PeriodicTimer<'a, T>(&'a T);

        impl<T> PeriodicTimer<'_, T>
        where
            T: Timer,
        {
            pub fn start(&self, _: MicrosDurationU64) -> Result<(), Infallible> {
                unimplemented!()
            }

            pub fn wait(&self) -> nb::Result<(), Infallible> {
                unimplemented!()
            }
        }
        pub trait Timer {}
    }
}

pub struct MicrosDurationU64;
