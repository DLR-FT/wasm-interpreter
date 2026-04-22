#![no_std]
#![no_main]

#[no_mangle]
fn main(mut x: i32, mut y: i32) -> i32 {
    x = x.wrapping_mul(x);

    y = x.wrapping_add(y);

    x = x.wrapping_mul(42);

    if (x + y) % 3 == 0 {
        171
    } else {
        if (x+y) % 3 == 1 {
            644
        } else {
            1310
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}