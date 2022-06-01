#![no_main]
#![no_std]

use nucleis as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART2])]
mod app {

    use stm32f4xx_hal::{
        gpio::gpioa::{PA10, PA9},
        gpio::{Alternate, PushPull},
        pac::USART1,
        prelude::*,
        serial::{config::Config as UartConfig, Serial},
        timer::{monotonic::MonoTimer, Timer},
    };
    // Copied from Pretty Hal:
    // https://github.com/jamesmunns/pretty-hal-machine/blob/f1a2ef7cbb62722ee9eb4f288da3559f9b11ad14/firmware/blackpill-phm/src/main.rs#L40
    // 7 is AF7 which is alternative function!
    type SandwichUart =
        Serial<USART1, (PA9<Alternate<PushPull, 7>>, PA10<Alternate<PushPull, 7>>), u8>;

    #[monotonic(binds = TIM2, default = true)]
    type Monotonic = MonoTimer<stm32f4xx_hal::pac::TIM2, 1_000_000>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
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

        let usart = Serial::new(
            device.USART1,
            (usart_tx, usart_rx),
            UartConfig::default().baudrate(9600.bps()),
            &clocks,
        )
        .unwrap();

        (Shared {}, Local { usart }, init::Monotonics(mono))
    }

    #[idle(local=[usart])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            if let Ok(d) = cx.local.usart.read() {
                defmt::debug!("Received byte from cx.local.usart.read(): {}", d);
                defmt::info!("Writing the byte back: cx.local.usart.write(d)");
                let _ = cx.local.usart.write(d);
            }
        }
    }
}
