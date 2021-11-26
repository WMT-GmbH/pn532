use clap::{App, Arg};
use std::time::Duration;

use pn532::Interface;
use pn532::{Pn532, Request};

use core::task::Poll;

use std::io::{Read, Write};

use linux_embedded_hal as hal;

const PROGRAM: Option<&'static str> = option_env!("CARGO_PKG_NAME");
const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const DESCRIPTION: Option<&'static str> = option_env!("CARGO_PKG_DESCRIPTION");

struct SerialPortInterface {
    pub port: Box<dyn serialport::SerialPort>,
}

impl Interface for SerialPortInterface {
    type Error = std::io::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        match self.port.write(frame) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        match self.port.read(buf) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

fn main() {
    env_logger::init();

    let matches = App::new(PROGRAM.unwrap_or("pn532-hsu-util"))
        .version(VERSION.unwrap_or("unknown"))
        .about(DESCRIPTION.unwrap_or(""))
        .arg(
            Arg::with_name("serial")
                .short("s")
                .long("serial")
                .value_name("DEVICE")
                .help("Serial Device for pn532")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let dev = matches.value_of("serial").unwrap();

    let serial = serialport::new(dev, 115200)
        .timeout(Duration::from_millis(500))
        .open()
        .expect("Failed to open port");

    let interface = SerialPortInterface { port: serial };

    let timer = hal::SysTimer::new();

    let mut pn532: Pn532<_, _, 32> = Pn532::new(interface, timer);

    if let Ok(fw) = pn532.process(
        &Request::GET_FIRMWARE_VERSION,
        4,
        Duration::from_millis(200),
    ) {
        println!("Firmware response: {:?}", fw);
    } else {
        println!("Unable to communicate with device.");
    }

    /*
    if let Ok(uid) = pn532.process(&Request::INLIST_ONE_ISO_A_TARGET, 7, Duration::from_millis(1000)){
        println!("Got uid: {:?}", uid);
        let result = pn532.process(&Request::ntag_read(10), 17, Duration::from_millis(50)).unwrap();
        if result[0] == 0x00 {
            println!("page 10: {:?}", &result[1..5]);
        }
    } else {
        println!("Failed to get iso target");
    }
    */
}
