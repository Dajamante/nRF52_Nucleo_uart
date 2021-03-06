//! The nRF52 is blinking the led of the Nucleo!
#![no_main]
#![no_std]

use nucleis as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART2])]
mod app {

    use stm32f4xx_hal::{
        gpio::gpioa::{PA10, PA5, PA9},
        gpio::{Alternate, Output, PushPull},
        pac::USART1,
        prelude::*,
        serial::{config::Config as UartConfig, Event, Serial},
        timer::{monotonic::MonoTimer, Timer},
    };

    type SandwichUart =
        Serial<USART1, (PA9<Alternate<PushPull, 7>>, PA10<Alternate<PushPull, 7>>), u8>;

    #[monotonic(binds = TIM2, default = true)]
    type Monotonic = MonoTimer<stm32f4xx_hal::pac::TIM2, 1_000_000>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
        usart: SandwichUart,
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
        let usart_rx = gpioa.pa10.into_alternate();
        let usart_tx = gpioa.pa9.into_alternate();
        let led = gpioa.pa5.into_push_pull_output();
        let mut usart = Serial::new(
            device.USART1,
            (usart_tx, usart_rx),
            UartConfig::default().baudrate(9600.bps()),
            &clocks,
        )
        .unwrap();
        usart.listen(Event::Rxne);
        (Shared {}, Local { usart, led }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    /// This task receives 0 or ones, and turns on or off the light accordingly!
    #[task(binds=USART1, local=[usart, led])]
    fn interupting(cx: interupting::Context) {
        if let Ok(d) = cx.local.usart.read() {
            match d {
                1 => {
                    defmt::debug!("Received {:?}, turning on the light!", d);
                    cx.local.led.set_high();
                    // echo back the byte
                    let _ = cx.local.usart.write(d);
                }
                0 => {
                    defmt::debug!("Received {:?}, turning on the light!", d);
                    cx.local.led.set_low();
                    let _ = cx.local.usart.write(d);
                }
                _ => defmt::debug!("Received noise, d = {:?}.", d),
            }
        }
    }
}
