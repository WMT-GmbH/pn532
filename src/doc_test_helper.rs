use core::convert::Infallible;
use core::time::Duration;

use embedded_hal::spi::{SpiDevice, Operation};
use embedded_hal::digital::OutputPin;
use crate::CountDown;

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
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl embedded_hal::digital::ErrorType for NoOpCS {
    type Error = Infallible;
}

impl SpiDevice for NoOpSPI
{
    fn transaction(&mut self, _operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl embedded_hal::spi::ErrorType for NoOpSPI{
    type Error = Infallible;
}

impl CountDown for NoOpTimer {
    type Time = Duration;
    type Error = nb::Error<void::Void>;

    fn start<T>(&mut self, _: T)
    where
        T: Into<Self::Time>,
    {
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        nb::Result::Ok(())
    }
}
