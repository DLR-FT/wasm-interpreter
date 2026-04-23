#![no_std]
#![no_main]

#[no_mangle]
fn main(mut x: i32) -> i32 {
    loop {
        if x < 0 {
            return 1
        }
        if x == 0 {
            return 2
        }
        if x == 1 {
            return 0
        }
        if x % 2 == 0 {
            x = x / 2;
        }
        else {
            x = x.wrapping_mul(3).wrapping_add(1);
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}