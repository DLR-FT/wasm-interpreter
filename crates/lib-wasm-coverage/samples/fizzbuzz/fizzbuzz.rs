#![no_std]
#![no_main]

#[no_mangle]
fn main(x: i32) -> i32 {
    let fizz = 171;
    let buzz = 644;
    let fizzbuzz = 1310;
    let silence = 379;
    if x % 15 == 0 {
        fizzbuzz
    } else {
        if x % 3 == 0 {
            fizz
        }
        else {
            if x % 5 == 0 {
                buzz
            }
            else {
                silence
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}