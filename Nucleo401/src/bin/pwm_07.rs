//! The nRF52 is dimming the light of the Nucleo.
//! ATM the dimmer function is bad, and need to be improved by making a function to dim, instead of magic numbers.
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

    // This is the Command that will be received instead of 0 or 1
    // It has been augmented with a Pwm() with an inner level
    // We use u16 because .get_max_duty() of pwm returns a u16.
    // But on the other side we sill use a u8 as it makes more sense for our function :)
    #[derive(Serialize, Deserialize, Format, Clone, Copy)]
    pub enum Command {
        On,
        Off,
        Pwm(u16),
    }
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        usart: SandwichUart,
        buf: Vec<u8, 16>,
        // pwm has now the led, they are inseparable!
        // aka: you cannont use the led as a peripheral now it is
        // owned by the pwm
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

        let buf = Vec::new();
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

    /// This task is a hardware task that does only dispatching
    /// And has highest priority.
    #[task(binds=USART1, priority = 2, local=[usart])]
    fn command_rx(cx: command_rx::Context) {
        while let Ok(d) = cx.local.usart.read() {
            parse::spawn(d).unwrap();
        }
    }

    /// This lower priority software task handles the message.
    /// With a terrible function to dim the light.
    /// Terrible but enough for proof of concept.
    #[task(capacity = 16, priority = 1, local=[pwm_channel, buf])]
    fn parse(cx: parse::Context, d: u8) {
        let _ = cx.local.buf.push(d);

        // 0 is the terminating byte of the Postcard serializer
        if d == 0 {
            if let Ok(command) = from_bytes_cobs(cx.local.buf) {
                defmt::debug!("Received complete command: {:?}.", command);
                match command {
                    Command::On => {
                        cx.local
                            .pwm_channel
                            .set_duty(cx.local.pwm_channel.get_max_duty());
                    }
                    Command::Off => {
                        cx.local.pwm_channel.set_duty(0);
                    }
                    Command::Pwm(level) => {
                        // 24 : this magic number corresponds to the max duty,
                        // nothing to worry about here.
                        // And division by zero is bad for health.
                        // the sent value is always max == 255
                        let max = cx.local.pwm_channel.get_max_duty();
                        if level > 10 && level < 250 {
                            defmt::info!(
                                "Duty = {:?}/{:?}",
                                cx.local.pwm_channel.get_max_duty(),
                                level * 8
                            );
                            cx.local.pwm_channel.set_duty(max / level)
                        } else if level >= 250 {
                            cx.local.pwm_channel.set_duty(0);
                        } else if level < 10 {
                            cx.local.pwm_channel.set_duty(max);
                        }
                    }
                }
            }
            //Clear också om from_bytes failar
            cx.local.buf.clear();
        };
    }
}
