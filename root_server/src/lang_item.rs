

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    panic!("sys_exit never returns!");
}
