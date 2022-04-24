[![Build](https://github.com/Dajamante/nucleo_playground/actions/workflows/build.yml/badge.svg)](https://github.com/Dajamante/nucleo_playground/actions/workflows/build.yml)

# Uart nRF52/STM32F401

The template of this project is done with: 

Based on https://github.com/knurling-rs/app-template

This repository is a collection of "pair" code between nRF52 (the sender) and STM32F401-Nucleo (the receiver). 

# Wiring

<p align="center">
<img src="./uarte1.JPG" width="60%">
<img src="./uarte5.JPG" width="60%">
<img src="./uarte6.JPG" width="60%">
</p>

* Nucleo D8/PA9 (tx) - nRF52 p1.07 (rx)
* Nucleo D2/PA10 (rx)- nRF52 p1.08 (tx)
* GND - GND

# How to run code

Open one terminal window *per* microcontroller and run:

```
cargo rb <project name>
```

Some programs are interracting, with the nRF52 as the sender and the Nucleo as the receiver.


|     | Interracting? | nRF52/Nucleo code   | What does it do?                                                                                                                                                                                |
| --- | ---------- | ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | No         | `minimal.rs`        | It says hi👋                                                                                                                                                                                    |
| 2   | No         | `blinky.rs`         | .. blinks a led💡                                                                                                                                                                               |
| 3   | yes        | `uarte.rs/usart.rs` | Sending byte beer emoji back and forth: <br /> `nRF52 says: look at this 🍻 we got back!`                                                                                                       |
| 4   | yes        | `lightning.rs`      | nRF52 is blinking the led of the nucleo 💡                                                                                                                                                      |
| 5   | yes        | `button.rs`         | nRF52 is blinking the led of the nucleo 💡, but with a button                                                                                                                                   |
| 6   | yes        | `postcard.rs`       | nRF52 is blinking the led of the nucleo 💡, with a [cobs](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing) command. Using [Postcard](https://docs.rs/postcard/0.7.3/postcard/)! |
| 7   | yes        | `pws.rs`            | nRF52 is dimming(*) the light of the nucleo 🔅💡🔅                                                                                                                                              |
| 8   | yes        | `interval.rs`       | nRF52 is blinking the light of the nucleo, with intervals. The light can be dimmed 🔅💡🔅.                                                                                                      |

*ATM the dimmer function is very bad, and need to be fixed (the incrementation must be based on a function, not magic numbers).

There is also an `interrupt.rs` file in the `Nucleo401` folder but I forgot what it was about ¯\_(ツ)_/¯. I did need an interrupt at some point.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.

[Knurling]: https://knurling.ferrous-systems.com
[Ferrous Systems]: https://ferrous-systems.com/
[GitHub Sponsors]: https://github.com/sponsors/knurling-rs
