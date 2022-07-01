//! The nRF52 is blinking the led of the Nucleo, but with a button!
#![no_main]
#![no_std]

use nucleis as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART2])]
mod app {
    use postcard::to_slice_cobs;
    use serde::Serialize;
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
    struct Shared {}

    #[local]
    struct Local {
        button: PC13<Input<PullUp>>,
        usart: SandwichUart,
    }

    // The Nucleo has only one button!
    #[derive(Serialize, defmt::Format)]
    pub enum Command {
        On,
        Off,
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
        (Shared {}, Local { usart, button }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(binds = EXTI15_10, priority=2, local = [button])]
    fn button_click(ctx: button_click::Context) {
        defmt::debug!("Button pushed");
        ctx.local.button.clear_interrupt_pending_bit();
        send::spawn_after(30.millis()).ok();
    }

    #[task(priority=1, local=[usart, is_on: bool = false])]
    fn send(cx: send::Context) {
        let mut buf = [0u8; 8];
        let mut cmd = Command::On;
        if *cx.local.is_on {
            cmd = Command::Off;
            *cx.local.is_on = false;
        } else {
            *cx.local.is_on = true;
        }
        defmt::info!("Command : {:?}", cmd);
        let data = to_slice_cobs(&cmd, &mut buf).unwrap();
        defmt::info!("Data : {:?}", data);

        for b in data.iter() {
            let _ = cx.local.usart.write(*b);
        }
        let _ = cx.local.usart.flush();
    }
}
