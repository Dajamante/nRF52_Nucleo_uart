//! The nRF52 is blinking the led of the Nucleo, but with a button!
#![no_main]
#![no_std]

use nucleis as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART2])]
mod app {
    use stm32f4xx_hal::{
        gpio::{
            gpioa::{PA10, PA5, PA9},
            gpioc::PC13,
        },
        gpio::{Alternate, Edge, Input, Output, PullUp, PushPull},
        pac::USART1,
        prelude::*,
        serial::{config::Config as UartConfig, Event, Serial},
        timer::{monotonic::MonoTimer, Timer},
    };

    // This is an alias to define USART, that needs to pins in alternate mode 7
    type SandwichUart =
        Serial<USART1, (PA9<Alternate<PushPull, 7>>, PA10<Alternate<PushPull, 7>>), u8>;

    // you need a monotonic clock. DWTSystick is the poor parent of clocks.
    #[monotonic(binds = TIM2, default = true)]
    type Monotonic = MonoTimer<stm32f4xx_hal::pac::TIM2, 1_000_000>;

    #[shared]
    struct Shared {
        button: PC13<Input<PullUp>>,
    }

    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
        usart: SandwichUart,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");
        let mut device = cx.device;

        // Set up the system clocks
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).require_pll48clk().freeze();
        let mut syscfg = device.SYSCFG.constrain();
        let mono = Timer::new(device.TIM2, &clocks).monotonic();
        let gpioa = device.GPIOA.split();
        let usart_rx = gpioa.pa10.into_alternate();
        let usart_tx = gpioa.pa9.into_alternate();
        let led = gpioa.pa5.into_push_pull_output();
        let gpioc = device.GPIOC.split();
        let mut button = gpioc.pc13.into_pull_up_input();

        button.make_interrupt_source(&mut syscfg);
        button.enable_interrupt(&mut device.EXTI);
        button.trigger_on_edge(&mut device.EXTI, Edge::Falling);

        let mut usart = Serial::new(
            device.USART1,
            (usart_tx, usart_rx),
            UartConfig::default().baudrate(9600.bps()),
            &clocks,
        )
        .unwrap();
        usart.listen(Event::Rxne);
        (
            Shared { button },
            Local { usart, led },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(binds = EXTI15_10, priority=2, shared = [button])]
    fn button_click(mut ctx: button_click::Context) {
        defmt::debug!("Button pushed");
        ctx.shared.button.lock(|b| b.clear_interrupt_pending_bit());
        send::spawn_after(25.millis()).ok();
    }

    #[task(priority=1, local=[usart, led, is_on : bool = false], shared=[button])]
    fn send(mut cx: send::Context) {
        let mut b = 0;
        if cx.shared.button.lock(|b| b.is_low()) {
            if *cx.local.is_on {
                b = 0;
                *cx.local.is_on = false;
            } else {
                b = 1;
                *cx.local.is_on = true;
            }
        }
        let _ = cx.local.usart.write(b);
        let _ = cx.local.usart.flush();
    }
}
