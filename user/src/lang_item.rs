

#[panic_handler]
fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
    panic!("sys_exit never returns!");
}
