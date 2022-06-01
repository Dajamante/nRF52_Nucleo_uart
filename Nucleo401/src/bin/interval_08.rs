//! The nRF52 is blinking the light of the nucleo, with intervals. The light can be dimmed.

#![no_main]
#![no_std]

use nucleis as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART2])]
mod app {
    use defmt::Format;
    use heapless::Vec;
    use postcard::from_bytes_cobs;

    use serde::{Deserialize, Serialize};
    use stm32f4xx_hal::{
        gpio::gpioa::{PA10, PA9},
        gpio::{Alternate, PushPull},
        pac::{TIM2, USART1},
        prelude::*,
        pwm::PwmChannel,
        serial::{config::Config as UartConfig, Event, Serial},
        timer::{monotonic::MonoTimer, Timer, C1},
    };

    type SandwichUart =
        Serial<USART1, (PA9<Alternate<PushPull, 7>>, PA10<Alternate<PushPull, 7>>), u8>;

    #[monotonic(binds = TIM5, default = true)]
    type Monotonic = MonoTimer<stm32f4xx_hal::pac::TIM5, 1_000_000>;

    #[derive(Serialize, Deserialize, Format, Clone, Copy)]

    pub enum Command {
        On,
        Off,
        Pwm(u16),
        Interval(u8),
    }
    #[shared]
    struct Shared {
        #[lock_free]
        brightness: u16,
        #[lock_free]
        time: u8,
    }

    #[local]
    struct Local {
        usart: SandwichUart,
        buf: Vec<u8, 16>,
        // pwm has now the led, they are inseparable!
        pwm_channel: PwmChannel<TIM2, C1>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");
        let device = cx.device;
        // Set up the system clocks
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).require_pll48clk().freeze();

        let mono = Timer::new(device.TIM5, &clocks).monotonic();
        let gpioa = device.GPIOA.split();
        let usart_rx = gpioa.pa10.into_alternate();
        let usart_tx = gpioa.pa9.into_alternate();
        let led = gpioa.pa5.into_alternate();
        let mut pwm_channel = Timer::new(device.TIM2, &clocks).pwm(led, 20.khz());
        pwm_channel.enable();

        let mut time = 1;
        let brightness = pwm_channel.get_max_duty();
        let buf = Vec::new();
        let mut usart = Serial::new(
            device.USART1,
            (usart_tx, usart_rx),
            UartConfig::default().baudrate(9600.bps()),
            &clocks,
        )
        .unwrap();
        blink::spawn();
        usart.listen(Event::Rxne);
        (
            Shared { brightness, time },
            Local {
                usart,
                pwm_channel,
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

    // A hardware task must do only the dispatching
    #[task(binds=USART1, priority = 2, local=[usart])]
    fn command_rx(cx: command_rx::Context) {
        while let Ok(d) = cx.local.usart.read() {
            parse::spawn(d).unwrap();
        }
    }

    // The lower priority software task handles the message
    #[task(capacity = 16, priority = 1, shared=[brightness, time], local=[buf])]
    fn parse(cx: parse::Context, d: u8) {
        let _ = cx.local.buf.push(d);

        // 0 is the terminating byte of the Postcard serializer
        if d == 0 {
            if let Ok(command) = from_bytes_cobs(cx.local.buf) {
                defmt::debug!("Received complete command: {:?}.", command);
                match command {
                    Command::On => {
                        *cx.shared.brightness = 255;
                    }
                    Command::Off => {
                        *cx.shared.brightness = 0;
                    }
                    Command::Pwm(level) => match level {
                        0..=10 => *cx.shared.brightness = 5,
                        11..=30 => *cx.shared.brightness = 20,
                        31..=80 => *cx.shared.brightness = 70,
                        81..=130 => *cx.shared.brightness = 110,
                        131..=170 => *cx.shared.brightness = 150,
                        171..=200 => *cx.shared.brightness = 180,
                        201..=230 => *cx.shared.brightness = 240,
                        231..=u16::MAX => *cx.shared.brightness = 255,
                    },
                    Command::Interval(sec) => *cx.shared.time = sec,
                }
            }
            //Clear också om from_bytes failar
            cx.local.buf.clear();
        };
    }

    #[task(shared=[brightness, time], local=[pwm_channel, powered: bool = false])]
    fn blink(cx: blink::Context) {
        let level = cx.shared.brightness;
        if *cx.local.powered {
            cx.local.pwm_channel.set_duty((*level * 8) as u16);
            *cx.local.powered = false;
        } else {
            cx.local.pwm_channel.set_duty(0);
            *cx.local.powered = true;
        }
        blink::spawn_after((*cx.shared.time as u32).secs()).ok();
    }
}
