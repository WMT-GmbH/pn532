# pn532
[<img alt="github" src="https://img.shields.io/badge/github-grey?style=for-the-badge&labelColor=555555&logo=github">](https://github.com/WMT-GmbH/pn532)
[<img alt="Crates.io" src="https://img.shields.io/crates/v/pn532?logo=rust&style=for-the-badge">](https://crates.io/crates/pn532)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-pn532-blue?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K">](https://docs.rs/pn532)

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
