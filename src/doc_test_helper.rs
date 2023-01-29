use core::convert::Infallible;
use core::time::Duration;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

pub struct NoOpCS;
pub struct NoOpBus;
pub struct NoOpTimer;

pub use eh::{get_async_pn532, get_pn532};

impl OutputPin for NoOpCS {
    type Error = Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Write<u8> for NoOpBus {
    type Error = Infallible;

    fn write(&mut self, _: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Transfer<u8> for NoOpBus {
    type Error = Infallible;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        Ok(words)
    }
}

#[cfg(not(feature = "eh1"))]
mod eh {
    use crate::spi::SPIInterface;

    /// used for doc tests
    pub fn get_pn532() -> Pn532<SPIInterface<NoOpBus, NoOpCS>, NoOpTimer> {
        Pn532::new(
            SPIInterface {
                spi: NoOpBus,
                cs: NoOpCS,
            },
            NoOpTimer,
        )
    }

    /// used for doc tests
    pub fn get_async_pn532() -> Pn532<SPIInterface<NoOpBus, NoOpCS>, ()> {
        Pn532::new(
            SPIInterface {
                spi: NoOpBus,
                cs: NoOpCS,
            },
            (),
        )
    }
}

#[cfg(feature = "eh1")]
mod eh {
    use core::convert::Infallible;

    use embedded_hal_1::i2c::{ErrorType, I2c, Operation, SevenBitAddress};

    use super::*;
    use crate::eh1::i2c::I2CInterface;
    use crate::Pn532;

    /// used for doc tests
    pub fn get_pn532() -> Pn532<I2CInterface<NoOpBus>, NoOpTimer> {
        Pn532::new(I2CInterface { i2c: NoOpBus }, NoOpTimer)
    }

    /// used for doc tests
    pub fn get_async_pn532() -> Pn532<I2CInterface<NoOpBus>, ()> {
        Pn532::new(I2CInterface { i2c: NoOpBus }, ())
    }

    impl I2c for NoOpBus {
        fn read(&mut self, _: SevenBitAddress, _: &mut [u8]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn write(&mut self, _: SevenBitAddress, _: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn write_iter<B>(&mut self, _: SevenBitAddress, _: B) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            Ok(())
        }

        fn write_read(
            &mut self,
            _: SevenBitAddress,
            _: &[u8],
            _: &mut [u8],
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn write_iter_read<B>(
            &mut self,
            _: SevenBitAddress,
            _: B,
            _: &mut [u8],
        ) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            Ok(())
        }

        fn transaction(
            &mut self,
            _: SevenBitAddress,
            _: &mut [Operation],
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn transaction_iter<'a, O>(&mut self, _: SevenBitAddress, _: O) -> Result<(), Self::Error>
        where
            O: IntoIterator<Item = Operation<'a>>,
        {
            Ok(())
        }
    }

    impl ErrorType for NoOpBus {
        type Error = Infallible;
    }
}

impl CountDown for NoOpTimer {
    type Time = Duration;

    fn start<T>(&mut self, _: T)
    where
        T: Into<Self::Time>,
    {
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        Ok(())
    }
}
