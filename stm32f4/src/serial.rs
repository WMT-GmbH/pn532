use crate::println;
use core::cell::RefCell;
use core::mem::MaybeUninit;
use critical_section::{CriticalSection, Mutex};
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

type Serial<'a> = usbd_serial::SerialPort<'a, UsbBus<USB>>;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];
static mut USB_BUS: MaybeUninit<UsbBusAllocator<UsbBus<USB>>> = MaybeUninit::uninit();
pub static SERIAL: Mutex<RefCell<Option<Serial>>> = Mutex::new(RefCell::new(None));

pub fn init(usb: USB) -> UsbDevice<'static, UsbBus<USB>> {
    let usb_bus = unsafe { USB_BUS.write(UsbBus::new(usb, &mut EP_MEMORY)) };

    let serial = usbd_serial::SerialPort::new(usb_bus);

    critical_section::with(|cs| {
        *SERIAL.borrow_ref_mut(cs) = Some(serial);
        println::set_print_func(cs, serial_print_func);
    });

    UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build()
}

fn serial_print_func(cs: CriticalSection, s: &str) -> core::fmt::Result {
    let mut serial_ref = SERIAL.borrow_ref_mut(cs);
    let serial = serial_ref.as_mut().unwrap();

    write(serial, s.as_bytes())
}

pub(crate) fn write(serial: &mut Serial, bytes: &[u8]) -> core::fmt::Result {
    let count = bytes.len();
    let mut write_offset = 0;
    while write_offset < count {
        match serial.write(&bytes[write_offset..count]) {
            Ok(len) => {
                write_offset += len;
            }
            Err(UsbError::WouldBlock) => {}
            Err(_) => return Err(core::fmt::Error),
        }
    }
    Ok(())
}
