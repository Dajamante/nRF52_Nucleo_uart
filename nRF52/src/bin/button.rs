#![no_main]
#![no_std]

use nrfie as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = nrf52840_hal::pac, dispatchers=[RADIO])]
mod app {

    use cortex_m::prelude::_embedded_hal_serial_Write as hal_write;
    use nrf52840_hal::{
        gpio::{p0::Parts as P0Parts, p1::Parts as P1Parts, Input, Pin, PullUp},
        gpiote::Gpiote,
        pac::{TIMER2, UARTE1},
        prelude::InputPin,
        uarte::{Baudrate, Parity, Pins as UartePins, Uarte, UarteTx},
    };
    use nrfie::mono::{ExtU32, MonoTimer};

    #[monotonic(binds = TIMER2, default = true)]
    type RticMono = MonoTimer<TIMER2>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        tx: UarteTx<UARTE1>,
        gpiote: Gpiote,
        btn_on: Pin<Input<PullUp>>,
        btn_off: Pin<Input<PullUp>>,
    }

    // Buffers are static when initiated there
    #[init(local=[
        uart_rx_buff: [u8;1] = [0;1],
        uart_tx_buff: [u8;1] = [0;1]
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

        (
            Shared {},
            Local {
                tx,
                btn_on,
                btn_off,
                gpiote,
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
            gpiote.reset_events();
            blink_led::spawn_after(30.millis()).ok();
        }
    }

    #[task(local=[tx, btn_on, btn_off])]
    fn blink_led(cx: blink_led::Context) {
        if cx.local.btn_on.is_low().unwrap() {
            defmt::info!("button on is pushed");
            let _ = hal_write::write(cx.local.tx, 1);
        } else if cx.local.btn_off.is_low().unwrap() {
            defmt::info!("button off is pushed");
            let _ = hal_write::write(cx.local.tx, 0);
        }
        // flush or tx.write(&[1])!
        let _ = cx.local.tx.flush();
    }
}
