// Copyright 2019 Adam Greig
// Dual licensed under the Apache 2.0 and MIT licenses.

use cortex_m::peripheral::SCB;

static mut FLAG: u32 = 0;
const FLAG_VALUE: u32 = 0xB00110AD;

/// Call this function at boot in pre_init, before statics are initialised.
///
/// If we reset due to requesting a bootload, this function will jump to
/// the system bootloader.
pub fn check() {
    unsafe {
        // If flag isn't set we just continue with the boot process
        if core::ptr::read_volatile(&FLAG) != FLAG_VALUE {
            return;
        }

        // Otherwise, clear the flag and jump to system bootloader
        core::ptr::write_volatile(&mut FLAG, 0);
        cortex_m::asm::bootload(0x1fff_0000 as *const u32);
    }
}

/// Call this function to trigger a reset into the system bootloader
pub fn enter() -> ! {
    unsafe {
        // Write flag value to FLAG
        core::ptr::write_volatile(&mut FLAG, FLAG_VALUE);
    }
    // Request system reset
    SCB::sys_reset();
}
