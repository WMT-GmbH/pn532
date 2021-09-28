use core::fmt::Debug;

use embedded_hal::timer::CountDown;

use crate::Interface;

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
    Timeout,
    InterfaceError(E),
}

impl<E: Debug> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::InterfaceError(e)
    }
}

pub struct Pn532<I>(pub I);

impl<I: Interface> Pn532<I> {
    // False positive: https://github.com/rust-lang/rust-clippy/issues/5787
    #[allow(clippy::needless_lifetimes)]
    pub async fn process_async<'a>(
        &mut self,
        frame: &[u8],
        response_buf: &'a mut [u8],
    ) -> Result<&'a [u8], Error<I::Error>> {
        self.0.write(frame)?;
        core::future::poll_fn(|_| self.0.wait_ready()).await?;
        self.receive_ack()?;
        core::future::poll_fn(|_| self.0.wait_ready()).await?;
        self.receive_response(frame[6], response_buf)
    }

    pub fn process<'a, T: CountDown>(
        &mut self,
        frame: &[u8],
        response_buf: &'a mut [u8],
        timeout: &mut T,
    ) -> Result<&'a [u8], Error<I::Error>> {
        self.0.write(frame)?;
        while self.0.wait_ready()?.is_pending() {
            if timeout.wait().is_ok() {
                return Err(Error::Timeout);
            }
        }
        self.receive_ack()?;
        while self.0.wait_ready()?.is_pending() {
            if timeout.wait().is_ok() {
                return Err(Error::Timeout);
            }
        }
        self.receive_response(frame[6], response_buf)
    }

    pub fn send(&mut self, frame: &[u8]) -> Result<(), Error<I::Error>> {
        self.0.write(frame)?;
        Ok(())
    }

    pub fn receive_ack(&mut self) -> Result<(), Error<I::Error>> {
        let mut ack_buf = [0; 6];
        self.0.read(&mut ack_buf)?;
        if ack_buf != ACK {
            Err(Error::NACK)
        } else {
            Ok(())
        }
    }

    /// frame[6]
    pub fn receive_response<'a>(
        &mut self,
        seventh_frame_byte: u8,
        response_buf: &'a mut [u8],
    ) -> Result<&'a [u8], Error<I::Error>> {
        self.0.read(response_buf)?;
        let expected_response_command = seventh_frame_byte + 1;
        parse_response(response_buf, expected_response_command)
    }
}

impl Pn532<()> {
    /// N = data.len() + 8
    pub const fn make_frame<const N: usize>(data: &[u8]) -> [u8; N] {
        if data.len() + 8 != N {
            panic!("N should be data.len() + 8");
        }

        let mut frame = [0; N];

        let frame_len = data.len() as u8 + 1; // data + frame identifier

        let mut data_sum = HOSTTOPN532; // sum(data + frame identifier)
        let mut i = 0;
        while i < data.len() {
            data_sum = data_sum.wrapping_add(data[i]);
            frame[6 + i] = data[i];
            i += 1;
        }

        const fn to_checksum(sum: u8) -> u8 {
            (!sum).wrapping_add(1)
        }

        frame[0] = PREAMBLE[0];
        frame[1] = PREAMBLE[1];
        frame[2] = PREAMBLE[2];
        frame[3] = frame_len;
        frame[4] = to_checksum(frame_len);
        frame[5] = HOSTTOPN532;
        frame[6 + data.len()] = to_checksum(data_sum);
        frame[7 + data.len()] = POSTAMBLE;
        frame
    }
}

fn parse_response<E: Debug>(
    response_buf: &[u8],
    expected_response_command: u8,
) -> Result<&[u8], Error<E>> {
    // TODO look for preamble and shift
    if response_buf[0..3] != PREAMBLE {
        return Err(Error::BadResponseFrame);
    }
    // Check length & length checksum
    let frame_len = response_buf[3];
    if frame_len < 2 || (frame_len.wrapping_add(response_buf[4])) != 0 {
        return Err(Error::BadResponseFrame);
    }
    match response_buf.get(5 + frame_len as usize) {
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
