//! The nRF52 is dimming the light of the Nucleo.
//! ATM the dimmer function is bad, and need to be improved by making a function to dim, instead of magic numbers.
#![no_main]
#![no_std]

use nrfie as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = nrf52840_hal::pac, dispatchers=[RADIO])]
mod app {
    use cortex_m::prelude::_embedded_hal_serial_Write as hal_write;
    use defmt::Format;
    use nrf52840_hal::{
        gpio::{p0::Parts as P0Parts, p1::Parts as P1Parts, Input, Pin, PullUp},
        gpiote::Gpiote,
        pac::{TIMER2, UARTE1},
        prelude::InputPin,
        uarte::{Baudrate, Parity, Pins as UartePins, Uarte, UarteTx},
    };
    use nrfie::mono::{ExtU32, MonoTimer};
    use postcard::to_slice_cobs;
    use serde::{Deserialize, Serialize};

    #[monotonic(binds = TIMER2, default = true)]
    type RticMono = MonoTimer<TIMER2>;
    #[derive(Serialize, Deserialize, Format, Clone, Copy)]
    // u8 is the most logical value, sends a pwm value between 0-255
    pub enum Command {
        On,
        Off,
        Pwm(u8),
    }
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        tx: UarteTx<UARTE1>,
        gpiote: Gpiote,
        btn_on: Pin<Input<PullUp>>,
        btn_off: Pin<Input<PullUp>>,
        bright_on: Pin<Input<PullUp>>,
        bright_off: Pin<Input<PullUp>>,
        pwm: u8,
    }

    // Buffers are static when initiated there
    #[init(local=[
        uart_rx_buff: [u8;1] = [0;1],
        uart_tx_buff: [u8;16] = [0;16]
    ])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");
        let device = cx.device;
        let timer = device.TIMER2;
        let mono = RticMono::new(timer);
        let p1 = P1Parts::new(device.P1);
        let p0 = P0Parts::new(device.P0);
        let btn_on = p0.p0_11.into_pullup_input().degrade();
        let btn_off = p0.p0_12.into_pullup_input().degrade();
        let bright_on = p0.p0_24.into_pullup_input().degrade();
        let bright_off = p0.p0_25.into_pullup_input().degrade();
        let pwm = 0;

        let txd = p1
            .p1_08
            .into_push_pull_output(nrf52840_hal::gpio::Level::High)
            .degrade();

        let rxd = p1.p1_07.into_floating_input().degrade();
        let pins = UartePins {
            rxd,
            txd,
            cts: None,
            rts: None,
        };

        let uarte = Uarte::new(device.UARTE1, pins, Parity::EXCLUDED, Baudrate::BAUD9600);
        let (tx, _rx) = uarte
            .split(cx.local.uart_tx_buff, cx.local.uart_rx_buff)
            .unwrap();
        let gpiote = Gpiote::new(device.GPIOTE);
        gpiote
            .channel0()
            .input_pin(&btn_on)
            .hi_to_lo()
            .enable_interrupt();
        gpiote
            .channel1()
            .input_pin(&btn_off)
            .hi_to_lo()
            .enable_interrupt();

        // Port or Channels can be used here
        gpiote.port().input_pin(&bright_on).low();
        gpiote.port().input_pin(&bright_off).low();
        // Enable interrupt for port event
        gpiote.port().enable_interrupt();

        (
            Shared {},
            Local {
                tx,
                btn_on,
                btn_off,
                gpiote,
                pwm,
                bright_on,
                bright_off,
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {}
    }

    #[task(binds=GPIOTE, local=[gpiote])]
    fn on_gpiote(cx: on_gpiote::Context) {
        let gpiote = cx.local.gpiote;
        if gpiote.channel0().is_event_triggered() || gpiote.channel1().is_event_triggered() {
            blink_led::spawn_after(15.millis()).ok();
        }
        if gpiote.port().is_event_triggered() {
            change_pwm::spawn_after(15.millis()).ok();
        }

        gpiote.reset_events();
    }
    #[task(local=[btn_on, btn_off])]
    fn blink_led(cx: blink_led::Context) {
        if cx.local.btn_on.is_low().unwrap() {
            send_command::spawn(Command::On).ok();
        } else if cx.local.btn_off.is_low().unwrap() {
            send_command::spawn(Command::Off).ok();
        }
    }

    #[task(local=[ pwm, bright_on, bright_off])]
    fn change_pwm(cx: change_pwm::Context) {
        if cx.local.bright_on.is_low().unwrap() && *cx.local.pwm < 255 {
            defmt::info!("pwm before saturating add sent : {:?}", *cx.local.pwm);
            *cx.local.pwm = (*cx.local.pwm).saturating_add(32);
            defmt::info!("pwm after saturating add sent : {:?}", *cx.local.pwm);
        } else if cx.local.bright_off.is_low().unwrap() && *cx.local.pwm > 0 {
            defmt::info!("pwm before saturating sub sent : {:?}", cx.local.pwm);
            *cx.local.pwm = (*cx.local.pwm).saturating_sub(32);

            defmt::info!("pwm after saturating sub sent : {:?}", *cx.local.pwm);
        }
        let cmd = Command::Pwm(*cx.local.pwm);
        send_command::spawn(cmd).ok();
    }

    #[task(local=[tx])]
    fn send_command(cx: send_command::Context, cmd: Command) {
        let mut buf = [0u8; 16];
        let data = to_slice_cobs(&cmd, &mut buf).unwrap();

        for b in data.iter() {
            let _ = cx.local.tx.write(*b);
            //defmt::info!("Byte sent : {:?}", *b);
        }
        let _ = cx.local.tx.flush();
    }
}
