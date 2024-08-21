use embedded_hal_async::{delay::DelayNs, i2c::I2c};

use crate::{Backlight, BitMode, Commands, CursorMoveDir, DisplayControl, DisplayShift, Font, Mode};

/// API to write to the LCD.
pub struct Lcd<'a, I, D>
where
    I: I2c,
    D: DelayNs,
{
    i2c: &'a mut I,
    address: u8,
    rows: u8,
    delay: &'a mut D,
    backlight_state: Backlight,
    cursor_on: bool,
    cursor_blink: bool,
    font_mode: Font,
}

impl<'a, I, D> Lcd<'a, I, D>
where
    I: I2c,
    D: DelayNs,
{
    /// Create new instance with only the I2C and delay instance.
    pub fn new(i2c: &'a mut I, delay: &'a mut D) -> Self {
        Self {
            i2c,
            delay,
            backlight_state: Backlight::On,
            address: 0,
            rows: 0,
            cursor_blink: false,
            cursor_on: false,
            font_mode: Font::Font5x8,
        }
    }

    /// Zero based number of rows.
    pub fn with_rows(mut self, rows: u8) -> Self {
        self.rows = rows;
        self
    }

    /// Set I2C address, see [lcd address].
    ///
    /// [lcd address]: https://badboi.dev/rust,/microcontrollers/2020/11/09/i2c-hello-world.html
    pub fn with_address(mut self, address: u8) -> Self {
        self.address = address;
        self
    }

    pub fn with_cursor_on(mut self, on: bool) -> Self {
        self.cursor_on = on;
        self
    }

    pub fn with_cursor_blink(mut self, blink: bool) -> Self {
        self.cursor_blink = blink;
        self
    }

    /// Initializes the hardware.
    ///
    /// Actual procedure is a bit obscure. This one was compiled from this [blog post],
    /// corresponding [code] and the [datasheet].
    ///
    /// [datasheet]: https://www.openhacks.com/uploadsproductos/eone-1602a1.pdf
    /// [code]: https://github.com/jalhadi/i2c-hello-world/blob/main/src/main.rs
    /// [blog post]: https://badboi.dev/rust,/microcontrollers/2020/11/09/i2c-hello-world.html
    pub async fn init(mut self) -> Result<Self, I::Error> {
        // Initial delay to wait for init after power on.
        self.delay.delay_ms(80).await;

        self.backlight(self.backlight_state).await?;

        // Init with 8 bit mode
        let mode_8bit = Mode::FunctionSet as u8 | BitMode::Bit8 as u8;
        self.write4bits(mode_8bit).await?;
        self.delay.delay_ms(5).await;
        self.write4bits(mode_8bit).await?;
        self.delay.delay_ms(5).await;
        self.write4bits(mode_8bit).await?;
        self.delay.delay_ms(5).await;

        // Switch to 4 bit mode
        let mode_4bit = Mode::FunctionSet as u8 | BitMode::Bit4 as u8;
        self.write4bits(mode_4bit).await?;

        self.update_function_set().await?;

        self.update_display_control().await?;
        self.command(Mode::Cmd as u8 | Commands::Clear as u8).await?; // Clear Display

        self.delay.delay_ms(2).await;

        // Entry right: shifting cursor moves to right
        self.command(Mode::EntrySet as u8 | CursorMoveDir::Left as u8 | DisplayShift::Decrement as u8 ).await?;
        self.return_home().await?;
        Ok(self)
    }

    async fn write4bits(&mut self, data: u8) -> Result<(), I::Error> {
        self.i2c.write(
            self.address,
            &[data | DisplayControl::Off as u8 | self.backlight_state as u8],
        ).await?;
        self.i2c.write(
            self.address,
            &[data | DisplayControl::DisplayOn as u8 | self.backlight_state as u8],
        ).await?;
        self.i2c.write(
            self.address,
            &[DisplayControl::Off as u8 | self.backlight_state as u8],
        ).await?;
        self.delay.delay_us(700).await;
        Ok(())
    }

    async fn send(&mut self, data: u8, mode: Mode) -> Result<(), I::Error> {
        let high_bits: u8 = data & 0xf0;
        let low_bits: u8 = (data << 4) & 0xf0;
        self.write4bits(high_bits | mode as u8).await?;
        self.write4bits(low_bits | mode as u8).await?;
        Ok(())
    }

    async fn command(&mut self, data: u8) -> Result<(), I::Error> {
        self.send(data, Mode::Cmd).await
    }

    pub async fn backlight(&mut self, backlight: Backlight) -> Result<(), I::Error> {
        self.backlight_state = backlight;
        self.i2c.write(
            self.address,
            &[DisplayControl::Off as u8 | backlight as u8],
        ).await
    }

    /// Write string to display.
    pub async fn write_str(&mut self, data: &str) -> Result<(), I::Error> {
        for c in data.chars() {
            self.send(c as u8, Mode::Data).await?;
        }
        Ok(())
    }

    /// Clear the display
    pub async fn clear(&mut self) -> Result<(), I::Error> {
        self.command(Commands::Clear as u8).await?;
        self.delay.delay_ms(2).await;
        Ok(())
    }

    /// Return cursor to upper left corner, i.e. (0,0).
    pub async fn return_home(&mut self) -> Result<(), I::Error> {
        self.command(Commands::ReturnHome as u8).await?;
        self.delay.delay_ms(2).await;
        Ok(())
    }

    /// Set the cursor to (rows, col). Coordinates are zero-based.
    pub async fn set_cursor(&mut self, row: u8, col: u8) -> Result<(), I::Error> {
        let shift: u8 = row * 0x40 + col;
        self.command(Mode::DDRAMAddr as u8 | shift).await
    }

    /// Recomputes display_ctrl and updates the lcd
    async fn update_display_control(&mut self) -> Result<(), I::Error> {
        let display_ctrl = if self.cursor_on {
            DisplayControl::DisplayOn as u8 | DisplayControl::CursorOn as u8
        } else {
            DisplayControl::DisplayOn as u8
        };
        let display_ctrl = if self.cursor_blink {
            display_ctrl | DisplayControl::CursorBlink as u8
        } else {
            display_ctrl
        };
        self.command(Mode::DisplayControl as u8 | display_ctrl).await
    }

    // Set if the cursor is blinking
    pub async fn cursor_blink(&mut self, blink: bool) -> Result<(), I::Error> {
        self.cursor_blink = blink;
        self.update_display_control().await
    }

    // Set the curser visibility
    pub async fn cursor_on(&mut self, on: bool) -> Result<(), I::Error> {
        self.cursor_on = on;
        self.update_display_control().await
    }

    /// Recomputes function set and updates the lcd
    async fn update_function_set(&mut self) -> Result<(), I::Error> {
        // Function set command
        let lines = if self.rows == 0 { 0x00 } else { 0x08 };
        self.command(
            Mode::FunctionSet as u8 |
            self.font_mode as u8 |
            lines, // Two line display
        ).await
    }

    /// Set the font mode used (5x8 or 5x10)
    pub async fn font_mode(&mut self, mode: Font) -> Result<(), I::Error> {
        self.font_mode = mode;
        self.update_function_set().await
    }

    /// Scrolls the display one char to the left
    pub async fn scroll_display_left(&mut self) -> Result<(), I::Error> {
        self.command(Commands::ShiftDisplayLeft as u8).await
    }

    /// Scrolls the display one char to the right
    pub async fn scroll_display_right(&mut self) -> Result<(), I::Error> {
        self.command(Commands::ShiftDisplayRight as u8).await
    }

    /// Scrolls the cursor one char to the left
    pub async fn scroll_cursor_left(&mut self) -> Result<(), I::Error> {
        self.command(Commands::ShiftCursorLeft as u8).await
    }

    /// Scrolls the cursor one char to the right
    pub async fn scroll_cursor_right(&mut self) -> Result<(), I::Error> {
        self.command(Commands::ShiftCursorRight as u8).await
    }
}