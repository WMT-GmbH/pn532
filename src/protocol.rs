use core::fmt::Debug;

use embedded_hal::timer::CountDown;

use crate::requests::Command;
use crate::{Interface, Request};

const PREAMBLE: [u8; 3] = [0x00, 0x00, 0xFF];
const POSTAMBLE: u8 = 0x00;
const ACK: [u8; 6] = [0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00];

const HOSTTOPN532: u8 = 0xD4;
const PN532TOHOST: u8 = 0xD5;

#[derive(Debug)]
pub enum Error<E: Debug> {
    NACK,
    BadResponseFrame,
    CrcError,
    BufTooSmall,
    TimeoutAck,
    TimeoutResponse,
    InterfaceError(E),
}

impl<E: Debug> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::InterfaceError(e)
    }
}

/// `N` >= max(response_len, request.data.len()) + 9
pub struct Pn532<I, T, const N: usize = 32> {
    pub interface: I,
    pub timer: T,
    buf: [u8; N],
}

impl<I: Interface, T: CountDown, const N: usize> Pn532<I, T, N> {
    pub fn process(
        &mut self,
        request: Request<'_>,
        response_len: usize,
        timeout: T::Time,
    ) -> Result<&[u8], Error<I::Error>> {
        self.timer.start(timeout);
        self.send(request)?;
        while self.interface.wait_ready()?.is_pending() {
            if self.timer.wait().is_ok() {
                return Err(Error::TimeoutAck);
            }
        }
        self.receive_ack()?;
        while self.interface.wait_ready()?.is_pending() {
            if self.timer.wait().is_ok() {
                return Err(Error::TimeoutResponse);
            }
        }
        self.receive_response(request.command, response_len)
    }
    pub fn process_no_response(
        &mut self,
        request: Request<'_>,
        timeout: T::Time,
    ) -> Result<(), Error<I::Error>> {
        self.timer.start(timeout);
        self.send(request)?;
        while self.interface.wait_ready()?.is_pending() {
            if self.timer.wait().is_ok() {
                return Err(Error::TimeoutAck);
            }
        }
        self.receive_ack()
    }
}
impl<I: Interface, T, const N: usize> Pn532<I, T, N> {
    pub fn new(interface: I, timer: T) -> Self {
        Pn532 {
            interface,
            timer,
            buf: [0; N],
        }
    }

    // False positive: https://github.com/rust-lang/rust-clippy/issues/5787
    #[allow(clippy::needless_lifetimes)]
    pub async fn process_async(
        &mut self,
        request: Request<'_>,
        response_len: usize,
    ) -> Result<&[u8], Error<I::Error>> {
        self.send(request)?;
        core::future::poll_fn(|_| self.interface.wait_ready()).await?;
        self.receive_ack()?;
        core::future::poll_fn(|_| self.interface.wait_ready()).await?;
        self.receive_response(request.command, response_len)
    }

    pub fn send(&mut self, request: Request<'_>) -> Result<(), Error<I::Error>> {
        let data_len = request.data.len();
        let frame_len = 2 + data_len as u8; // frame identifier + command + data

        let mut data_sum = HOSTTOPN532.wrapping_add(request.command as u8); // sum(command + data + frame identifier)
        for &byte in request.data {
            data_sum = data_sum.wrapping_add(byte);
        }

        const fn to_checksum(sum: u8) -> u8 {
            (!sum).wrapping_add(1)
        }

        self.buf[0] = PREAMBLE[0];
        self.buf[1] = PREAMBLE[1];
        self.buf[2] = PREAMBLE[2];
        self.buf[3] = frame_len;
        self.buf[4] = to_checksum(frame_len);
        self.buf[5] = HOSTTOPN532;
        self.buf[6] = request.command as u8;

        self.buf[7..7 + data_len].copy_from_slice(request.data);

        self.buf[7 + data_len] = to_checksum(data_sum);
        self.buf[8 + data_len] = POSTAMBLE;

        self.interface.write(&self.buf[..9 + data_len])?;
        Ok(())
    }

    pub fn receive_ack(&mut self) -> Result<(), Error<I::Error>> {
        let mut ack_buf = [0; 6];
        self.interface.read(&mut ack_buf)?;
        if ack_buf != ACK {
            Err(Error::NACK)
        } else {
            Ok(())
        }
    }

    pub fn receive_response(
        &mut self,
        sent_command: Command,
        response_len: usize,
    ) -> Result<&[u8], Error<I::Error>> {
        let response_buf = &mut self.buf[..response_len + 9];
        response_buf.fill(0); // zero out buf
        self.interface.read(response_buf)?;
        let expected_response_command = sent_command as u8 + 1;
        parse_response(response_buf, expected_response_command)
    }

    /// Send an ACK frame to force the PN532 to abort the current process.
    /// In that case, the PN532 discontinues the last processing and does not answer anything
    /// to the host controller.
    /// Then, the PN532 starts again waiting for a new command.
    pub fn abort(&mut self) -> Result<(), Error<I::Error>> {
        self.interface.write(&ACK)?;
        Ok(())
    }
}

fn parse_response<E: Debug>(
    response_buf: &[u8],
    expected_response_command: u8,
) -> Result<&[u8], Error<E>> {
    if response_buf[0..3] != PREAMBLE {
        return Err(Error::BadResponseFrame);
    }
    // Check length & length checksum
    let frame_len = response_buf[3];
    if frame_len < 2 || (frame_len.wrapping_add(response_buf[4])) != 0 {
        return Err(Error::BadResponseFrame);
    }
    match response_buf.get(5 + frame_len as usize + 1) {
        None => {
            return Err(Error::BufTooSmall);
        }
        Some(&POSTAMBLE) => {}
        Some(_) => {
            return Err(Error::BadResponseFrame);
        }
    }

    if response_buf[5] != PN532TOHOST || response_buf[6] != expected_response_command {
        return Err(Error::BadResponseFrame);
    }
    // Check frame checksum value matches bytes
    let checksum = response_buf[5..5 + frame_len as usize + 1]
        .iter()
        .fold(0u8, |s, &b| s.wrapping_add(b));
    if checksum != 0 {
        return Err(Error::CrcError);
    }
    // Adjust response buf and return it
    Ok(&response_buf[7..5 + frame_len as usize])
}
