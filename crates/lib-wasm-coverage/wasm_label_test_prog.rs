#![no_main]
#![no_std]
#![feature(asm_experimental_arch)]
#[allow(named_asm_labels)]
#[inline(never)]
#[unsafe(no_mangle)]
fn entry(mut x: i32) -> i32 {
    if x % 2 == 0 {
        // Comment this line and see the branch disappear
        unsafe { core::arch::asm!("my_complex_label:") };
        unsafe { core::arch::asm!("nop") };
        x += 3;
    }
    unsafe { core::arch::asm!("xd:") };
    unsafe { core::arch::asm!("nop") };
    x += 5;
    return x;
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
