#![no_main]
#![no_std]

use nucleis as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART2])]
mod app {
    use defmt::Format;
    use heapless::Vec;
    use postcard::{from_bytes_cobs, to_slice_cobs};
    use serde::{Deserialize, Serialize};
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

    #[derive(Serialize, Deserialize, Format, Clone, Copy)]
    pub enum Command {
        On,
        Off,
    }
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
        usart: SandwichUart,
        buf: Vec<u8, 3>,
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
        let buf = Vec::new();
        let mut usart = Serial::new(
            device.USART1,
            (usart_tx, usart_rx),
            UartConfig::default().baudrate(9600.bps()),
            &clocks,
        )
        .unwrap();
        usart.listen(Event::Rxne);
        (Shared {}, Local { usart, led, buf }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(binds=USART1, priority = 2, local=[usart])]
    fn command_rx(cx: command_rx::Context) {
        if let Ok(d) = cx.local.usart.read() {
            parse::spawn(d).ok();
        }
    }

    #[task(capacity = 16, priority = 1, local=[led, buf])]
    fn parse(cx: parse::Context, d: u8) {
        defmt::debug!("cx.local.buf: {:?}.", cx.local.buf.as_slice());

        let _ = cx.local.buf.push(d);
        // terminating byte
        if d == 0 {
            if let Ok(command) = from_bytes_cobs(cx.local.buf) {
                match command {
                    Command::On => {
                        defmt::debug!("Received {:?}!", command);
                        cx.local.led.set_high();
                        cx.local.buf.clear();
                    }
                    Command::Off => {
                        defmt::debug!("Received {:?}!", command);
                        cx.local.led.set_low();
                        cx.local.buf.clear();
                    }
                }
            }
            //Clear också om from_bytes failar
            cx.local.buf.clear();
        };
    }
}
