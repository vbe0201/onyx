#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod arch;

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn main(kernel_base: *const u8, kernel_layout: *const u8, kip1_base: *const u8) {
    unsafe {
        core::arch::asm!(
            "li a0, 'B'",
            "li a1, 0x10000000",
            "sw a0, 0(a1)",
            options(nostack)
        );
    }
}
