#![no_main]
#![no_std]

use nucleis as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1])]
mod app {

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");

        task1::spawn().ok();

        // Setup the monotonic timer
        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        defmt::info!("idle");

        loop {
            continue;
        }
    }

    #[task]
    fn task1(_cx: task1::Context) {
        defmt::info!("Hello from our minimal format!");
    }
}
