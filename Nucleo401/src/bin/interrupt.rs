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
        usart: SandwichUart,
        counter: usize,
        buf: [u8; 4],
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
        let counter = 0;
        let buf = [0_u8; 4];
        let mut usart = Serial::new(
            device.USART1,
            (usart_tx, usart_rx),
            UartConfig::default().baudrate(9600.bps()),
            &clocks,
        )
        .unwrap();
        usart.listen(Event::Rxne);
        (
            Shared {},
            Local {
                usart,
                counter,
                buf,
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(binds=USART1, local=[usart, counter, buf])]
    fn interupting(cx: interupting::Context) {
        if let Ok(d) = cx.local.usart.read() {
            cx.local.buf[*cx.local.counter] = d;
            *cx.local.counter += 1;
            //defmt::info!("Writing the byte back: cx.local.usart.write(d)");
            let _ = cx.local.usart.write(d);

            if *cx.local.counter == 4 {
                if let Ok(text) = core::str::from_utf8(&cx.local.buf[..]) {
                    defmt::debug!("ST-nucleo got a {:?}", text);
                    //defmt::debug!("{}", cx.local.buf);
                    *cx.local.counter = 0;
                    *cx.local.buf = [0, 0, 0, 0];
                }
            }
        }
    }
}
