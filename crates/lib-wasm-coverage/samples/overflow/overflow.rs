#![no_std]
#![no_main]

#[no_mangle]
fn main(x: i32, y: i32) -> i32 {
    let z = x.wrapping_add(y);
    if z < 0 {
        67
    }
    else {
        2310
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}