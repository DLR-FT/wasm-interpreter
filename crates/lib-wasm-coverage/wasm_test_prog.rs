#![no_std]

#[no_mangle]
fn entry(mut x: i32) -> i32 {
    x = x.wrapping_mul(2);

    x = x.wrapping_add(5);

    x = x.wrapping_mul(3);

    if x % 2 == 0 {
        1
    } else {
        2
    }
}
