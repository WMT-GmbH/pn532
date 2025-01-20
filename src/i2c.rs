//! I2C interfaces
use core::fmt::Debug;

#[cfg(feature = "is_sync")]
use core::convert::Infallible;
#[cfg(feature = "is_sync")]
use core::task::Poll;
#[cfg(feature = "is_sync")]
use embedded_hal::digital::InputPin;
#[cfg(feature = "is_sync")]
use embedded_hal::i2c::I2c as embedded_I2c;

#[cfg(not(feature = "is_sync"))]
use embedded_hal_async::i2c::I2c as embedded_I2c;

use crate::Interface;
use embedded_hal::i2c::{Error, ErrorKind, NoAcknowledgeSource, Operation};

/// To be used in `Interface::wait_ready` implementations
pub const PN532_I2C_READY: u8 = 0x01;

/// I2C address of the Pn532
pub const I2C_ADDRESS: u8 = 0x24;

/// I2C Interface without IRQ pin
#[derive(Clone, Debug)]
pub struct I2CInterface<I2C>
where
    I2C: embedded_I2c,
{
    pub i2c: I2C,
}

#[maybe_async::maybe_async(AFIT)]
impl<I2C> Interface for I2CInterface<I2C>
where
    I2C: embedded_I2c,
{
    type Error = I2C::Error;

    async fn write(&mut self, frame: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write(I2C_ADDRESS, frame).await
    }

    async fn wake_up(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    // wait_ready implementations differ between sync / async

    #[maybe_async::sync_impl]
    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        let mut buf = [0];
        if let Err(e) = self.i2c.read(I2C_ADDRESS, &mut buf) {
            // It's possible that the PN532 does not ACK the read request when it is not ready.
            // See https://github.com/WMT-GmbH/pn532/issues/4 for more info
            return match e.kind() {
                ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address)
                | ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown) => Poll::Pending,
                _ => Poll::Ready(Err(e)),
            };
        }

        if buf[0] == PN532_I2C_READY {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
    #[maybe_async::async_impl]
    async fn wait_ready(&mut self) -> Result<(), Self::Error> {
        let mut buf = [0];
        while buf[0] != PN532_I2C_READY {
            if let Err(e) = self.i2c.read(I2C_ADDRESS, &mut buf).await {
                // It's possible that the PN532 does not ACK the read request when it is not ready.
                // See https://github.com/WMT-GmbH/pn532/issues/4 for more info
                match e.kind() {
                    ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address)
                    | ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown) => (),
                    _ => return Err(e),
                };
            }
        }
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c
            .transaction(
                I2C_ADDRESS,
                &mut [Operation::Read(&mut [0]), Operation::Read(buf)],
            )
            .await
    }
}

/// I2C Interface with IRQ pin
#[maybe_async::sync_impl]
#[derive(Clone, Debug)]
#[maybe_async::sync_impl]
pub struct I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: embedded_hal::i2c::I2c,
    IRQ: InputPin<Error = Infallible>,
{
    pub i2c: I2C,
    pub irq: IRQ,
}

#[maybe_async::sync_impl]
impl<I2C, IRQ> Interface for I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: embedded_hal::i2c::I2c,
    IRQ: InputPin<Error = Infallible>,
{
    type Error = <I2C as embedded_hal::i2c::ErrorType>::Error;

    fn wake_up(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write(&mut self, frame: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write(I2C_ADDRESS, frame)
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        // infallible unwrap because of IRQ bound
        if self.irq.is_low().unwrap() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.transaction(
            I2C_ADDRESS,
            &mut [Operation::Read(&mut [0]), Operation::Read(buf)],
        )
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use embedded_hal::digital::ErrorType;
    use embedded_hal_mock::eh1::digital::Transaction as DigitalTransaction;
    use embedded_hal_mock::eh1::digital::{Mock as DigitalMock, State};
    use embedded_hal_mock::eh1::i2c::Mock as I2cMock;
    use embedded_hal_mock::eh1::i2c::Transaction as I2cTransaction;

    #[test]
    fn test_i2c() {
        let mut i2c = I2CInterface {
            i2c: I2cMock::new(&[
                // write
                I2cTransaction::write(I2C_ADDRESS, vec![1, 2]),
                // wait_ready
                I2cTransaction::read(I2C_ADDRESS, vec![0x00]),
                I2cTransaction::read(I2C_ADDRESS, vec![0x00])
                    .with_error(ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address)),
                I2cTransaction::read(I2C_ADDRESS, vec![0x01]),
                // read
                I2cTransaction::transaction_start(I2C_ADDRESS),
                I2cTransaction::read(I2C_ADDRESS, vec![0]),
                I2cTransaction::read(I2C_ADDRESS, vec![3, 4]),
                I2cTransaction::transaction_end(I2C_ADDRESS),
            ]),
        };

        i2c.write(&mut [1, 2]).unwrap();

        assert_eq!(i2c.wait_ready(), Poll::Pending);
        assert_eq!(i2c.wait_ready(), Poll::Pending);
        assert_eq!(i2c.wait_ready(), Poll::Ready(Ok(())));

        let mut buf = [0, 0];
        i2c.read(&mut buf).unwrap();
        assert_eq!(buf, [3, 4]);

        i2c.i2c.done();
    }

    /// Wrapper around `DigitalMock` that is "infallible"
    pub struct PinMock {
        pub mock: DigitalMock,
    }

    impl PinMock {
        pub fn new(transactions: &[DigitalTransaction]) -> Self {
            Self {
                mock: DigitalMock::new(transactions),
            }
        }
    }

    impl ErrorType for PinMock {
        type Error = Infallible;
    }

    impl InputPin for PinMock {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            self.mock.is_high().map_err(|e| panic!("{:?}", e))
        }

        fn is_low(&mut self) -> Result<bool, Self::Error> {
            self.mock.is_low().map_err(|e| panic!("{:?}", e))
        }
    }

    #[test]
    fn test_i2c_with_irq() {
        let mut i2c = I2CInterfaceWithIrq {
            i2c: I2cMock::new(&[
                // write
                I2cTransaction::write(I2C_ADDRESS, vec![1, 2]),
                // read
                I2cTransaction::transaction_start(I2C_ADDRESS),
                I2cTransaction::read(I2C_ADDRESS, vec![0]),
                I2cTransaction::read(I2C_ADDRESS, vec![3, 4]),
                I2cTransaction::transaction_end(I2C_ADDRESS),
            ]),
            irq: PinMock::new(&[
                DigitalTransaction::get(State::High),
                DigitalTransaction::get(State::Low),
            ]),
        };

        i2c.write(&mut [1, 2]).unwrap();

        assert_eq!(i2c.wait_ready(), Poll::Pending);
        assert_eq!(i2c.wait_ready(), Poll::Ready(Ok(())));

        let mut buf = [0, 0];
        i2c.read(&mut buf).unwrap();
        assert_eq!(buf, [3, 4]);

        i2c.i2c.done();
        i2c.irq.mock.done();
    }
}
