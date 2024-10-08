#![no_std]
//! Driver to write characters to LCD displays with a LM1602 connected via i2c like [this one] with
//! 16x2 characters. It requires a I2C instance implementing [`embedded_hal::blocking::i2c::Write`]
//! and a instance to delay execution with [`embedded_hal::blocking::delay::DelayMs`].
//! Other LCD sizes are supported, up to displays with 20x4 characters. Everything that uses a
//! HD44780U or comparable controller and is connected via i2c should work
//!
//! Usage:
//! ```
//! const LCD_ADDRESS: u8 = 0x27; // Address depends on hardware, see link below
//!
//! // Create a I2C instance, needs to implement embedded_hal::blocking::i2c::Write, this
//! // particular uses the arduino_hal crate for avr microcontrollers like the arduinos.
//! let dp = arduino_hal::Peripherals::take().unwrap();
//! let pins = arduino_hal::pins!(dp);
//! let mut i2c = arduino_hal::I2c::new(
//!     dp.TWI, //
//!     pins.a4.into_pull_up_input(), // use respective pins
//!     pins.a5.into_pull_up_input(),
//!     50000,
//! );
//! let mut delay = arduino_hal::Delay::new();
//!
//! let mut lcd = lcd_lcm1602_i2c::LCD16x2::new(&mut i2c, &mut delay)
//!     .with_address(LCD_ADDRESS)
//!     .with_cursor_on(false) // no visible cursor
//!     .init().unwrap();
//! ```
//!
//! This [site][lcd address] describes how to find the address of your LCD devices.
//!
//! [this one]: https://funduinoshop.com/elektronische-module/displays/lcd/16x02-i2c-lcd-modul-hintergrundbeleuchtung-blau
//! [lcd address]: https://www.ardumotive.com/i2clcden.html

use sync_lcd::Lcd;

#[cfg(feature = "async")]
pub mod async_lcd;
pub mod sync_lcd;

pub enum DisplayControl {
    Off = 0x00,
    CursorBlink = 0x01,
    CursorOn = 0x02,
    DisplayOn = 0x04,
}

#[derive(Copy, Clone)]
pub enum Backlight {
    Off = 0x00,
    On = 0x08,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum Mode {
    Cmd = 0x00,
    Data = 0x01,
    EntrySet = 0x04,
    DisplayControl = 0x08,
    FunctionSet = 0x20,
    DDRAMAddr = 0x80,
}

enum Commands {
    Clear = 0x01,
    ReturnHome = 0x02,
    ShiftCursorLeft = 16,
    ShiftCursorRight = 16 | 4,
    ShiftDisplayLeft = 16 | 8,
    ShiftDisplayRight = 16 | 8 | 4,
}

enum BitMode {
    Bit4 = 0x0 << 4,
    Bit8 = 0x1 << 4,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Font {
    Font5x8 = 0x00,
    Font5x10 = 0x04,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum CursorMoveDir {
    Right = 0x00,
    Left = 0x02,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum DisplayShift {
    Decrement = 0x00,
    Increment = 0x01,
}

// offsets taken from the NewLiquidCrystal library
const OFFSETS_NORMAL: [u8; 4] = [0x00, 0x40, 0x14, 0x54]; // For regular LCDs
const OFFSETS_16X4: [u8; 4] = [0x00, 0x40, 0x10, 0x50]; // For 16x4 LCDs

pub type LCD16x2<'a, I, D> = Lcd<'a, 2, 16, I, D>;
pub type LCD16x4<'a, I, D> = Lcd<'a, 4, 16, I, D>;
pub type LCD20x4<'a, I, D> = Lcd<'a, 4, 20, I, D>;

#[cfg(feature = "async")]
pub type AsyncLCD16x2<'a, I, D> = async_lcd::Lcd<'a, 2, 16, I, D>;
#[cfg(feature = "async")]
pub type AsyncLCD16x4<'a, I, D> = async_lcd::Lcd<'a, 4, 16, I, D>;
#[cfg(feature = "async")]
pub type AsyncLCD20x4<'a, I, D> = async_lcd::Lcd<'a, 4, 20, I, D>;
