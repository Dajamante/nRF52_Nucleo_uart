#![no_main]
#![no_std]

use nucleis as _; // global logger + panicking-behavior + memory layout

//! This is the second mini-project. It is independant from it's nRF52 counterpart.
#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1])]
mod app {

    use stm32f4xx_hal::{
        gpio::gpioa::PA5,
        gpio::{Output, PushPull},
        prelude::*,
        timer::{
            monotonic::{ExtU32, MonoTimer},
            Timer,
        },
    };

    #[monotonic(binds = TIM2, default = true)]
    type Monotonic = MonoTimer<stm32f4xx_hal::pac::TIM2, 1_000_000>;

    #[shared]
    struct Shared {}
    #[local]
    struct Local {
        // We do not use the Pin type, but PA5! : stm32f4xx-hal has no degrade () for the pins as nrf.
        led: PA5<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");
        let device = cx.device;

        // Set up the system clocks
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).require_pll48clk().freeze();

        let mono = Timer::new(device.TIM2, &clocks).monotonic();

        let gpioa = device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();
        blink::spawn().ok();
        (Shared {}, Local { led }, init::Monotonics(mono))
    }
    #[idle]
    fn idle(_: idle::Context) -> ! {
        defmt::info!("idle");

        loop {
            continue;
        }
    }

    #[task(local=[led])]
    fn blink(cx: blink::Context) {
        cx.local.led.toggle();
        defmt::info!("Printing from blinky");
        blink::spawn_after(5.secs()).ok();
    }
}
