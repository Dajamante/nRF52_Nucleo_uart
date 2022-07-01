//! The Nucleo is blinking the led of the nRF52, but with a button!
#![no_main]
#![no_std]

use nrfie as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = nrf52840_hal::pac, dispatchers=[RADIO])]
mod app {
    use defmt::Format;
    use heapless::Vec;
    use nrf52840_hal::prelude::OutputPin;
    use nrf52840_hal::prelude::_embedded_hal_serial_Read;
    use nrf52840_hal::uarte::UarteRx;
    use nrf52840_hal::{
        gpio::{p0::Parts as P0Parts, p1::Parts as P1Parts, Level, Output, Pin, PushPull},
        pac::{TIMER2, UARTE1},
        uarte::{Baudrate, Parity, Pins as UartePins, Uarte},
    };
    use nrfie::mono::{ExtU32, MonoTimer};
    use postcard::from_bytes_cobs;
    use serde::{Deserialize, Serialize};

    #[monotonic(binds = TIMER2, default = true)]
    type RticMono = MonoTimer<TIMER2>;

    #[shared]
    struct Shared {
        led: Pin<Output<PushPull>>,
    }

    #[local]
    struct Local {
        rx: UarteRx<UARTE1>,
        buf: Vec<u8, 3>,
    }
    // This is the Command that will be sent instead of 0 or 1
    #[derive(Serialize, Format, Deserialize, Clone, Copy)]
    pub enum Command {
        On,
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
        let buf = Vec::new();
        let p0 = P0Parts::new(device.P0);
        let p1 = P1Parts::new(device.P1);

        // We need it for initialisation
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
        let mut led = p0.p0_13.into_push_pull_output(Level::High).degrade();

        let uarte = Uarte::new(device.UARTE1, pins, Parity::EXCLUDED, Baudrate::BAUD9600);
        let (_tx, rx) = uarte
            .split(cx.local.uart_tx_buff, cx.local.uart_rx_buff)
            .unwrap();

        (Shared { led }, Local { buf, rx }, init::Monotonics(mono))
    }

    #[idle(local=[rx, buf])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            if let Ok(d) = cx.local.rx.read() {
                let _ = cx.local.buf.push(d);
                if d == 0 {
                    if let Ok(command) = from_bytes_cobs(cx.local.buf) {
                        match command {
                            Command::On => {
                                defmt::debug!("Received {:?} 🟢 , turning on the light!", command);
                                cx.local.buf.clear();
                                light_on::spawn_after(25.millis()).ok();
                            }
                            _ => defmt::debug!("Received nothing"),
                        }
                    }
                }
            }
        }
    }

    #[task(shared=[led])]
    fn light_on(mut cx: light_on::Context) {
        let _ = cx.shared.led.lock(|l| l.set_high());
        defmt::debug!("Light on");
        light_off::spawn_after(2.secs()).ok();
    }

    #[task(shared=[led])]
    fn light_off(mut cx: light_off::Context) {
        defmt::debug!("Light off");
        let _ = cx.shared.led.lock(|l| l.set_low());
    }
}
