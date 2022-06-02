//! The nRF52 is blinking the led of the Nucleo!
#![no_main]
#![no_std]

use nrfie as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = nrf52840_hal::pac, dispatchers=[RADIO])]
mod app {

    use cortex_m::prelude::_embedded_hal_serial_Write as hal_write;
    use nrf52840_hal::gpio::PushPull;
    use nrf52840_hal::{
        gpio::{p0::Parts as P0Parts, p1::Parts as P1Parts, Level, Output, Pin},
        pac::{TIMER2, UARTE1},
        prelude::OutputPin,
        uarte::{Baudrate, Parity, Pins as UartePins, Uarte, UarteRx, UarteTx},
    };
    use nrfie::mono::{ExtU32, MonoTimer};

    #[monotonic(binds = TIMER2, default = true)]
    type RticMono = MonoTimer<TIMER2>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        rx: UarteRx<UARTE1>,
        tx: UarteTx<UARTE1>,
        blink: usize,
        led: Pin<Output<PushPull>>,
    }

    // Buffers are static when initiated here!
    #[init(local=[
        uart_rx_buff: [u8;1] = [0;1],
        uart_tx_buff: [u8;4] = [0;4]
    ])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");
        let device = cx.device;
        let timer = device.TIMER2;
        let mono = RticMono::new(timer);
        let p1 = P1Parts::new(device.P1);
        let p0 = P0Parts::new(device.P0);
        let blink = 0;
        let led = p0.p0_13.into_push_pull_output(Level::High).degrade();

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
        let (tx, rx) = uarte
            .split(cx.local.uart_tx_buff, cx.local.uart_rx_buff)
            .unwrap();
        sending_blink::spawn().ok();

        (
            Shared {},
            Local { tx, rx, blink, led },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    /// This function toggles the blink variable every second.
    /// It also sends the corresponding instruction to the Nucleo!
    #[task(local=[rx, tx, blink, led])]
    fn sending_blink(cx: sending_blink::Context) {
        if *cx.local.blink == 1 {
            let _ = hal_write::write(cx.local.tx, 0);
            *cx.local.blink = 0;
            let _ = cx.local.led.set_high();
        } else {
            let _ = hal_write::write(cx.local.tx, 1);
            *cx.local.blink = 1;
            let _ = cx.local.led.set_low();
        }
        sending_blink::spawn_after(1.secs()).ok();
    }
}
