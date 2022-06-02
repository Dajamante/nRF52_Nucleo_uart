//! In this mini-project, you will send beer back and forth 4 bytes that are converted to a beer emoji 🍻.
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
    // We create a generic serial type that supports u8 and u16 (albeit the later is not used). 7 is AF7 which is alternative function!
    // https://rhye.org/post/stm32-with-opencm3-1-usart-and-printf/#:~:text=In%20addition%20to%20acting%20as,Timer%2C%20DMA%20or%20other%20peripherals.
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

    /// Idle is always on, and is monitoring received bytes.
    /// When 4 bytes arrived, we convert to a beer emoji with core::str::from_utf8()
    /// And then, a beer is send back, byte by byte!
    #[idle(local=[usart, buf: [u8;4] = [0;4], counter: usize = 0])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            if let Ok(d) = cx.local.usart.read() {
                cx.local.buf[*cx.local.counter] = d;
                *cx.local.counter += 1;

                if *cx.local.counter == 4 {
                    if let Ok(beer) = core::str::from_utf8(&cx.local.buf[..]) {
                        defmt::info!("Oh look a {} ", beer);
                        *cx.local.buf = [0_u8; 4];
                        *cx.local.counter = 0;
                    }
                }

                defmt::info!("Received byte from cx.local.usart.read(): {}", d);
                defmt::info!("Writing the byte back: cx.local.usart.write(d)");
                let _ = cx.local.usart.write(d);
            }
        }
    }
}
