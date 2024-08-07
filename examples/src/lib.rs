#![allow(non_upper_case_globals)]

use {
    anyhow::Result,
    bitflags::bitflags,
    crossterm::{
        cursor,
        event::{self, Event, KeyCode},
        execute, style, terminal,
    },
    std::{fmt, io::stdout, mem::ManuallyDrop, time::Duration},
};

#[macro_export]
macro_rules! yeet {
    ($e:expr) => {
        return Err($e.into())
    };
}

pub const NUM_COLUMNS: u16 = 50;
pub const NUM_ROWS: u16 = 25;

bitflags! {
    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    pub struct Buttons: u16 {
        const Action1 = 0x1 << 0;
        const Action2 = 0x1 << 1;
        const Action3 = 0x1 << 2;
        const Action4 = 0x1 << 3;
        const Left = 0x1 << 4;
        const Right = 0x1 << 5;
        const Up = 0x1 << 6;
        const Down = 0x1 << 7;
        const More = 0x1 << 8;
        const Quit = 0x1 << 9;
    }
}

impl Buttons {
    pub fn name(self) -> &'static str {
        match self {
            Buttons::Action1 => "1",
            Buttons::Action2 => "2",
            Buttons::Action3 => "3",
            Buttons::Action4 => "4",
            Buttons::Left => "LEFT",
            Buttons::Right => "RIGHT",
            Buttons::Up => "UP",
            Buttons::Down => "DOWN",
            Buttons::More => "SPACE",
            Buttons::Quit => "ESCAPE",
            _ => "Unknown",
        }
    }

    fn translate(vkey: KeyCode) -> Self {
        match vkey {
            KeyCode::Char('1') => Buttons::Action1,
            KeyCode::Char('2') => Buttons::Action2,
            KeyCode::Char('3') => Buttons::Action3,
            KeyCode::Char('4') => Buttons::Action4,
            KeyCode::Left => Buttons::Left,
            KeyCode::Right => Buttons::Right,
            KeyCode::Up => Buttons::Up,
            KeyCode::Down => Buttons::Down,
            KeyCode::Char(' ') => Buttons::More,
            KeyCode::Esc => Buttons::Quit,
            _ => Buttons::empty(),
        }
    }
}

pub struct Example {
    pressed: Buttons,
    down: Buttons,
    buffer: String,
    ypos: usize,
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

impl Example {
    pub fn init() -> Result<Example> {
        std::fs::remove_file("example.log").ok();
        let file_log = tracing_appender::rolling::never(".", "example.log");
        let (writer, _guard) = tracing_appender::non_blocking(file_log);
        tracing_subscriber::fmt()
            .pretty()
            .with_ansi(false)
            .with_writer(writer)
            .with_env_filter("info,fmod::handle=trace".parse::<tracing_subscriber::EnvFilter>()?)
            .init();

        std::panic::set_hook(Box::new(tracing_panic::panic_hook));

        terminal::enable_raw_mode()?;
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::SetTitle("FMOD Example"),
        )?;

        tracing::info!("example initialized");

        Ok(Example {
            pressed: Buttons::empty(),
            down: Buttons::empty(),
            buffer: String::with_capacity(((NUM_COLUMNS + 1) * NUM_ROWS) as usize),
            ypos: 0,
            _guard,
        })
    }

    pub fn close(self) -> Result<()> {
        let Example { .. } = &mut *ManuallyDrop::new(self);
        execute!(stdout(), terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

impl Drop for Example {
    fn drop(&mut self) {
        execute!(stdout(), terminal::LeaveAlternateScreen).ok();
        terminal::disable_raw_mode().ok();
    }
}

impl Example {
    pub fn update(&mut self) -> Result<()> {
        let prev_buttons = self.down;
        while event::poll(Duration::ZERO)? {
            if let Event::Key(event) = event::read()? {
                match event.kind {
                    event::KeyEventKind::Press => self.down |= Buttons::translate(event.code),
                    event::KeyEventKind::Release => self.down &= !Buttons::translate(event.code),
                    _ => (),
                }
            }
        }
        self.pressed = (prev_buttons ^ self.down) & self.down;
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::Purge),
            cursor::MoveTo(0, 0),
            style::Print(&self.buffer),
        )?;
        self.buffer.clear();
        self.ypos = 0;
        Ok(())
    }

    fn draw_internal(&mut self, text: &str) {
        if self.ypos < NUM_ROWS as usize {
            self.ypos += 1;
            use std::fmt::Write;
            writeln!(
                &mut self.buffer,
                "{text:NUM_COLUMNS$.NUM_COLUMNS$}",
                NUM_COLUMNS = NUM_COLUMNS as usize
            )
            .unwrap()
        }
    }

    pub fn draw(&mut self, text: impl fmt::Display) {
        let string = text.to_string();
        let mut string = &*string;

        loop {
            let mut consume_nl = false;
            let mut copy_len = string.len();
            if let Some(nl) = string.find('\n') {
                consume_nl = true;
                copy_len = nl;
            }
            if copy_len > NUM_COLUMNS as usize {
                // hard wrap
                copy_len = NUM_COLUMNS as usize;
                // soft wrap
                if let Some(space) = string[..copy_len].rfind(' ') {
                    copy_len = space;
                }
            }
            self.draw_internal(&string[..copy_len]);
            string = &string[copy_len + consume_nl as usize..];

            if string.is_empty() {
                break;
            }
        }
    }

    pub fn btn_press(&self, btn: Buttons) -> bool {
        self.pressed.contains(btn)
    }

    pub fn btn_down(&self, btn: Buttons) -> bool {
        self.down.contains(btn)
    }
}

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

#[macro_export]
macro_rules! media {
    ($fname:expr) => {
        ::fmod::cstr8!(concat!(env!("CARGO_MANIFEST_DIR"), "/media/", $fname))
    };
}
