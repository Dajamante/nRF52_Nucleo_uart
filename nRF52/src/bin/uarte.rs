#![no_main]
#![no_std]

use nrfie as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = nrf52840_hal::pac, dispatchers=[RADIO])]
mod app {
    use nrf52840_hal::prelude::_embedded_hal_blocking_serial_Write;
    use nrf52840_hal::prelude::_embedded_hal_serial_Read;
    use nrf52840_hal::{
        gpio::p1::Parts,
        pac::{TIMER2, UARTE1},
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
        counter: usize,
        buf: [u8; 4],
    }

    // Buffers are static when initiated there
    #[init(local=[
        uart_tx_buff: [u8; 4] = [0;4],
        uart_rx_buff: [u8; 1] = [0;1]
        ]
    )]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");
        let device = cx.device;
        let timer = device.TIMER2;
        let mono = RticMono::new(timer);
        let p1 = Parts::new(device.P1);
        let counter = 0;
        let buf = [0; 4];
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
        sending_buffer::spawn().ok();

        (
            Shared {},
            Local {
                tx,
                rx,
                counter,
                buf,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(local=[rx, counter, buf])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            if let Ok(byte) = cx.local.rx.read() {
                //let counter = *cx.local.counter;
                //let mut buf = *cx.local.buf;
                cx.local.buf[*cx.local.counter] = byte;
                *cx.local.counter += 1;
                //defmt::info!("We received back an: {}", cx.local.buf);
                if *cx.local.counter == 4 {
                    if let Ok(beer) = core::str::from_utf8(&cx.local.buf[..]) {
                        defmt::info!("nRF52 says: look at this {} we got back!", beer);
                        *cx.local.buf = [0, 0, 0, 0];
                        *cx.local.counter = 0;
                    }
                }
            }
        }
    }

    #[task(local=[tx])]
    fn sending_buffer(cx: sending_buffer::Context) {
        let mut buf = [0u8; 4];
        let beer: [u8; 4] = [0xf0, 0x9f, 0x8d, 0xbb];
        buf.copy_from_slice(&beer);
        let _ = cx.local.tx.bwrite_all(&buf);
        sending_buffer::spawn_after(1.secs()).ok();
    }
}
