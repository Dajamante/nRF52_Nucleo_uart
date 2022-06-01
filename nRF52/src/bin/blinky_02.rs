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
    // DwtSystic is an emergency solution if
    // you just want something that works everywhere
    // but it has more overhead and poorer accuracy
    #[monotonic(binds = TIMER2, default = true)]
    type RticMono = MonoTimer<TIMER2>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
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
        // Wrapped in a Result, but this is just reading to a register
        if cx.local.led1.is_set_high().unwrap() {
            // can this go wrong? No! We just set a register :)
            let _ = cx.local.led1.set_low();
            let _ = cx.local.led2.set_high();
        } else {
            let _ = cx.local.led1.set_high();
            let _ = cx.local.led2.set_low();
        }

        blink::spawn_after(10.secs()).ok();
    }
}
