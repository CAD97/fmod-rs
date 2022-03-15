//! Yes, this is super messy. It's a port of the FMOD Core exmaple's common.cpp
//! and common_platform.cpp with minmal adjustment.

#![allow(non_upper_case_globals)]

use {
    bitflags::bitflags,
    crossterm::{
        cursor,
        event::{self, Event, KeyCode},
        execute, style, terminal,
    },
    once_cell::sync::Lazy,
    std::{
        fmt,
        io::stdout,
        sync::{
            atomic::{AtomicBool, AtomicU16, Ordering},
            Mutex,
        },
        time::Duration,
    },
};

pub const NUM_COLUMNS: u16 = 50;
pub const NUM_ROWS: u16 = 25;

bitflags! {
    #[derive(Default)]
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

pub fn init() -> crossterm::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(
        stdout(),
        terminal::SetSize(NUM_COLUMNS, NUM_ROWS),
        cursor::Hide,
        terminal::SetTitle("FMOD Example"),
    )?;
    Ok(())
}

pub fn close() -> crossterm::Result<()> {
    terminal::disable_raw_mode()?;
    Ok(())
}

static PRESSED: Lazy<Mutex<Buttons>> = Lazy::new(Default::default);
static DOWN: Lazy<Mutex<Buttons>> = Lazy::new(Default::default);
static PAUSED: AtomicBool = AtomicBool::new(false);
static BUFFER: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new(String::with_capacity((NUM_COLUMNS * NUM_ROWS) as usize)));
static YPOS: AtomicU16 = AtomicU16::new(0);

pub fn update() -> crossterm::Result<()> {
    let mut new_buttons = Buttons::empty();
    while event::poll(Duration::ZERO)? {
        if let Event::Key(event) = event::read()? {
            match event.code {
                KeyCode::Char('1') => new_buttons |= Buttons::Action1,
                KeyCode::Char('2') => new_buttons |= Buttons::Action2,
                KeyCode::Char('3') => new_buttons |= Buttons::Action3,
                KeyCode::Char('4') => new_buttons |= Buttons::Action4,
                KeyCode::Left => new_buttons |= Buttons::Left,
                KeyCode::Right => new_buttons |= Buttons::Right,
                KeyCode::Up => new_buttons |= Buttons::Up,
                KeyCode::Down => new_buttons |= Buttons::Down,
                KeyCode::Char(' ') => new_buttons |= Buttons::More,
                KeyCode::Esc => new_buttons |= Buttons::Quit,
                KeyCode::F(1) => {
                    PAUSED.fetch_nand(true, Ordering::SeqCst);
                }
                _ => (),
            }
        }
    }
    {
        let mut pressed = PRESSED.lock().unwrap();
        let mut down = DOWN.lock().unwrap();
        *pressed = (*down ^ new_buttons) & new_buttons;
        *down = new_buttons;
    }
    let mut buffer = BUFFER.lock().unwrap();
    if !PAUSED.load(Ordering::SeqCst) {
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            terminal::EnableLineWrap,
            style::Print(&buffer),
        )?;
    }
    buffer.clear();
    YPOS.store(0, Ordering::SeqCst);
    Ok(())
}

pub fn sleep(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

pub fn draw_text(text: &str) {
    if YPOS.fetch_add(1, Ordering::SeqCst) < NUM_ROWS {
        use std::fmt::Write;
        let mut buffer = BUFFER.lock().unwrap();
        write!(
            &mut buffer,
            "{text:NUM_COLUMNS$.NUM_COLUMNS$}",
            NUM_COLUMNS = NUM_COLUMNS as usize
        )
        .unwrap();
    }
}

pub fn btn_press(btn: Buttons) -> bool {
    PRESSED.lock().unwrap().contains(btn)
}

pub fn btn_down(btn: Buttons) -> bool {
    DOWN.lock().unwrap().contains(btn)
}

pub fn btn_str(btn: Buttons) -> &'static str {
    match btn {
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

#[macro_export]
macro_rules! media {
    ($fname:expr) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../media/", $fname)
    };
}

pub fn draw(text: impl fmt::Display) {
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
        draw_text(&string[..copy_len]);
        string = &string[copy_len + consume_nl as usize..];

        if string.is_empty() {
            break;
        }
    }
}
