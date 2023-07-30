# ut61e

This crate provides an interface to [UNI-T UT61E Digital Multimeters](https://meters.uni-trend.com/product/ut61-series/).

This is a port of [ut61e_py](https://github.com/4x1md/ut61e_py).

Currently, this crate requires my fork of the [`cp211x_uart` crate](https://crates.io/crates/cp211x_uart); my fork:

- uses the [`hidapi` crate](https://crates.io/crates/hidapi), instead of the unmaintained [`hid` crate](https://crates.io/crates/hid).
- fixes several bugs that either result in panics or truncated data

#### License

<sup>
Licensed under <a href="LICENSE-MIT">MIT license</a>.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in ut61e by you shall be
licensed as above, without any additional terms or conditions.
</sub>