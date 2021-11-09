# pn532
[<img alt="github" src="https://img.shields.io/badge/github-grey?style=for-the-badge&labelColor=555555&logo=github">](https://github.com/WMT-GmbH/pn532)

`no_std` implementation of the `Pn532` protocol using `embedded_hal` traits.

Since communication with the Pn532 can be rather slow at times,
communication can be split into multiple parts, a timeout can be provided or an async runtime
can be used.

The Pn532 supports different serial links. The `Interface` trait abstracts
over these different links.

`Interface` can be manually implemented or one the four provided interface structs can be used:
* `spi::SPIInterface`
* `spi::SPIInterfaceWithIrq`
* `i2c::I2CInterface`
* `i2c::I2CInterfaceWithIrq`

## SPI example
```rust
use pn532::{Pn532, Request};
use pn532::spi::SPIInterface;

// spi, cs and timer are structs implementing their respective embedded_hal traits.

let interface = SPIInterface {
    spi,
    cs,
};
let mut pn532: Pn532<_, _, 32> = Pn532::new(interface, timer);
if let Ok(uid) = pn532.process(&Request::INLIST_ONE_ISO_A_TARGET, 7, 1000.ms()){
    let result = pn532.process(&Request::ntag_read(10), 17, 50.ms()).unwrap();
    if result[0] == 0x00 {
        println!("page 10: {:?}", &result[1..5]);
    }
}
```

## `msb-spi` feature
If you want to use either `spi::SPIInterface` or `spi::SPIInterfaceWithIrq` and
your peripheral cannot be set to **lsb mode** you need to enable the `msb-spi` feature of this crate.

#### License
<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
