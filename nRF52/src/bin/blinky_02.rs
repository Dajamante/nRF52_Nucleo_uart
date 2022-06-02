//! This is the second mini-project. It is independant from it's nucleo counterpart.
#![no_main]
#![no_std]

use nrfie as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = nrf52840_hal::pac, dispatchers=[UARTE1])]
mod app {

    use nrf52840_hal::{
        gpio::{p0::Parts, Level, Output, Pin, PushPull},
        pac::TIMER2,
        prelude::{OutputPin, StatefulOutputPin},
    };

    use nrfie::mono::{ExtU32, MonoTimer};

    // Monotonic is a timer that never stops with a fixed tick rate.
    // Its type is "read only". RTIC is agnostic and has a trait called
    // Monotonic that the program needs to implement.
    // We need to use a timer that implements this trait,
    // that we will use for everything time-related
    #[monotonic(binds = TIMER2, default = true)]
    type RticMono = MonoTimer<TIMER2>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        // We DO use a Pin type, because we can degrade them later.
        // Contarily to the STM32F that needs to use the specific pins.
        led1: Pin<Output<PushPull>>,
        led2: Pin<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");
        let device = cx.device;
        let timer = device.TIMER2;
        let mono = RticMono::new(timer);
        let p0 = Parts::new(device.P0);
        // Here we use degrade, the equivalent in stm32f4xx is erase.
        // It's generally called "type erasure" as in - you are erasing part of the type information, and storing it as data at runtime
        // https://github.com/stm32-rs/stm32f4xx-hal/blob/cb7cd4e4ad63a0b72e6711b720b1d29ff2db3063/src/gpio.rs#L326-L342
        let led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();
        let led2 = p0.p0_14.into_push_pull_output(Level::High).degrade();
        blink::spawn().ok();

        // Setup the monotonic timer
        (Shared {}, Local { led1, led2 }, init::Monotonics(mono))
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        defmt::info!("idle");
        loop {
            continue;
        }
    }

    #[task(local=[led1, led2])]
    fn blink(cx: blink::Context) {
        // is_set_high() is wrapped in a Result, but this is just reading to a register.
        if cx.local.led1.is_set_high().unwrap() {
            // As you see, a result is ignored.
            // Can this go wrong? No! We just set a register :)
            let _ = cx.local.led1.set_low();
            let _ = cx.local.led2.set_high();
        } else {
            let _ = cx.local.led1.set_high();
            let _ = cx.local.led2.set_low();
        }

        blink::spawn_after(10.secs()).ok();
    }
}
