#![no_std]
#![no_main]

mod arch;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn main() {}
