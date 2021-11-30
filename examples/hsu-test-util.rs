//! Usage:
//! cargo run --example hsu-test-util --features std -- --serial <DEVICE>

use std::time::Duration;

use clap::{App, Arg};

use pn532::serialport::{SerialPortInterface, SysTimer};
use pn532::{Pn532, Request};

const PROGRAM: Option<&'static str> = option_env!("CARGO_PKG_NAME");
const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const DESCRIPTION: Option<&'static str> = option_env!("CARGO_PKG_DESCRIPTION");

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

    let timer = SysTimer::new();

    let mut pn532: Pn532<_, _, 32> = Pn532::new(interface, timer);

    pn532.interface.send_wakeup_message().unwrap(); // required for HSU

    if let Ok(fw) = pn532.process(
        &Request::GET_FIRMWARE_VERSION,
        4,
        Duration::from_millis(200),
    ) {
        println!("Firmware response: {:?}", fw);
    } else {
        println!("Unable to communicate with device.");
    }

    // TODO: Implement other functionality:
    // * list capabilities
    // * list cards
    // * wait for NFC event
}
