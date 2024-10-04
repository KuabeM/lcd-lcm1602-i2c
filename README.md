# LCD LM1602 I2C driver

Driver to write characters to LCD displays with a LM1602 connected via i2c like [this one] with
16x2 characters. It requires a I2C instance implementing [`embedded_hal::blocking::i2c::Write`]
and a instance to delay execution with [`embedded_hal::blocking::delay::DelayMs`].
Other LCD sizes are supported, up to displays with 20x4 characters. Everything that uses a
HD44780U or comparable controller and is connected via i2c should work.
Like [this]

Usage:
```
const LCD_ADDRESS: u8 = 0x27; // Address depends on hardware, see link below

// Create a I2C instance, needs to implement embedded_hal::blocking::i2c::Write, this
// particular uses the arduino_hal crate for avr microcontrollers like the arduinos.
let dp = arduino_hal::Peripherals::take().unwrap();
let pins = arduino_hal::pins!(dp);
let mut i2c = arduino_hal::I2c::new(
    dp.TWI, //
    pins.a4.into_pull_up_input(), // use respective pins
    pins.a5.into_pull_up_input(),
    50000,
);
let mut delay = arduino_hal::Delay::new();

let mut lcd = lcd_lcm1602_i2c::LCD16x2::new(&mut i2c, &mut delay)
    .address(LCD_ADDRESS)
    .cursor_on(false) // no visible cursor
    .init().unwrap();
```

This [site][lcd address] describes how to find the address of your LCD devices.

There is a similar crate [lcd_1602_i2c] but that did not work with [this display][this one].

[this one]: https://funduinoshop.com/elektronische-module/displays/lcd/16x02-i2c-lcd-modul-hintergrundbeleuchtung-blau
[lcd address]: https://www.ardumotive.com/i2clcden.html
[lcd_1602_i2c]: https://crates.io/crates/lcd_1602_i2c
[this]: https://www.az-delivery.de/en/products/hd44780-2004-lcd-display-bundle-4x20-zeichen-mit-i2c-schnittstelle
