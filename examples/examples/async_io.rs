/*==============================================================================
//! Async IO Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2022.
//!
//! This example shows how to play a stream and use a custom file handler that
//! defers reads for the streaming part. FMOD will allow the user to return
//! straight away from a file read request and supply the data at a later time.
==============================================================================*/

use {
    fmod_examples::{media, sleep_ms, Buttons, Example, NUM_COLUMNS, NUM_ROWS},
    once_cell::sync::Lazy,
    std::{
        collections::VecDeque,
        fmt::{self, Write},
        fs::File,
        io::{ErrorKind, Read, Seek, SeekFrom},
        pin::Pin,
        sync::{
            atomic::{AtomicBool, Ordering::SeqCst},
            Mutex,
        },
        thread,
    },
};

static ASYNC_LIST: Lazy<Mutex<VecDeque<fmod::AsyncReadInfo<File>>>> = Lazy::new(Default::default);
static THREAD_QUIT: AtomicBool = AtomicBool::new(false);
static SLEEP_BREAK: AtomicBool = AtomicBool::new(false);

// A little text buffer to allow a scrolling window
const DRAW_ROWS: u16 = NUM_ROWS - 8;
const DRAW_COLS: u16 = NUM_COLUMNS;
static LINE_DATA: Lazy<Mutex<String>> = Lazy::new(|| {
    Mutex::new(
        format!("{:NUM_COLUMNS$}\n", "", NUM_COLUMNS = NUM_COLUMNS as usize)
            .repeat(DRAW_ROWS as usize),
    )
});

fn add_line(line: impl fmt::Display) {
    let mut line_data = LINE_DATA.lock().unwrap();
    line_data.drain(0..DRAW_COLS as usize + 1);
    let line = line.to_string();
    let copy_len = line.len().min(DRAW_COLS as usize);
    writeln!(
        &mut line_data,
        "{:DRAW_COLS$.DRAW_COLS$}",
        &line[..copy_len],
        DRAW_COLS = DRAW_COLS as usize,
    )
    .unwrap();
}

fn draw_lines(example: &mut Example) {
    let line_data = LINE_DATA.lock().unwrap();
    for line in line_data.lines() {
        example.draw(line);
    }
}

// File callbacks
enum MyFileSystem {}

unsafe impl fmod::file::FileSystem for MyFileSystem {
    type File = File;

    fn open(name: &std::ffi::CStr) -> fmod::Result<(u32, Pin<Box<File>>)> {
        let name = name.to_str().map_err(|_| fmod::Error::FileNotFound)?;
        let file = Box::pin(File::open(name).map_err(|_| fmod::Error::FileNotFound)?);
        let meta = file.metadata().map_err(|_| fmod::Error::FileBad)?;
        let size = meta.len().try_into().map_err(|_| fmod::Error::FileBad)?;
        Ok((size, file))
    }

    fn close(file: Pin<Box<File>>) -> fmod::Result {
        file.sync_all().map_err(|_| fmod::Error::FileBad)
    }
}

unsafe impl fmod::file::SyncFileSystem for MyFileSystem {
    fn read(
        mut file: Pin<&mut File>,
        buffer: &mut [std::mem::MaybeUninit<u8>],
    ) -> fmod::Result<u32> {
        // SAFETY: 😬
        let mut buf = unsafe { &mut *(buffer as *mut _ as *mut [u8]) };

        while !buf.is_empty() {
            match file.read(buf) {
                Ok(0) => break,
                Ok(n) => buf = &mut buf[n..],
                Err(e) if e.kind() == ErrorKind::Interrupted => {},
                Err(_) => return Err(fmod::Error::FileBad),
            }
        }

        Ok((buffer.len() - buf.len()) as u32)
    }

    fn seek(mut file: Pin<&mut File>, pos: u32) -> fmod::Result {
        let pos = pos as u64;
        let sought = file
            .seek(SeekFrom::Start(pos))
            .map_err(|_| fmod::Error::FileCouldNotSeek)?;
        if sought == pos {
            Ok(())
        } else {
            Err(fmod::Error::FileBad)
        }
    }
}

unsafe impl fmod::file::AsyncFileSystem for MyFileSystem {
    unsafe fn read(info: fmod::AsyncReadInfo<Self::File>) -> fmod::Result {
        let mut list = ASYNC_LIST.lock().unwrap();

        add_line(format_args!(
            "REQUEST {:5} bytes, offset {:5} PRIORITY = {}.",
            info.size(),
            info.offset(),
            info.priority(),
        ));
        list.push_back(info);

        // Example only: Use your native filesystem scheduler / priority here
        if info.priority() > 50 {
            SLEEP_BREAK.store(true, SeqCst);
        }

        Ok(())
    }

    unsafe fn cancel(info: fmod::AsyncReadInfo<Self::File>) -> fmod::Result {
        let mut list = ASYNC_LIST.lock().unwrap();

        // Find the pending IO request and remove it
        for (i, data) in list.iter().copied().enumerate() {
            if data == info {
                list.remove(i);
                // Signal FMOD to wake up, this operation has been cancelled.
                info.done(Err(fmod::Error::FileDiskEjected));
                return Err(fmod::Error::FileDiskEjected);
            }
        }

        // IO request not found, it must have completed already
        Ok(())
    }
}

/// Async file IO processing thread
fn process_queue() {
    'main: while !THREAD_QUIT.load(SeqCst) {
        let info = ASYNC_LIST.lock().unwrap().pop_front();

        if let Some(info) = info {
            unsafe {
                // Example only: Demonstration of priority influencing
                // turnaround time
                for _ in 0..50 {
                    sleep_ms(10);
                    if SLEEP_BREAK.load(SeqCst) {
                        add_line("URGENT REQUEST - reading now!");
                        SLEEP_BREAK.store(false, SeqCst);
                        break;
                    }
                }

                // Process the seek and read request with EOF handling
                let mut file = info.handle_mut();
                match file.seek(SeekFrom::Start(info.offset() as u64)) {
                    Ok(_) => {},
                    Err(_) => {
                        info.done(Err(fmod::Error::FileCouldNotSeek));
                        continue 'main;
                    },
                }

                // SAFETY: 😬
                let mut buf = &mut (*(info.buffer() as *mut [u8]));
                while !buf.is_empty() {
                    match file.read(buf) {
                        Ok(0) => break,
                        Ok(n) => buf = &mut buf[n..],
                        Err(e) if e.kind() == ErrorKind::Interrupted => {},
                        Err(_) => {
                            info.done(Err(fmod::Error::FileBad));
                            continue 'main;
                        },
                    }
                }

                let did_read = info.size() - buf.len() as u32;
                if !buf.is_empty() {
                    add_line(format_args!(
                        "FED     {:5} bytes, offset {:5} (* EOF)",
                        did_read,
                        info.offset(),
                    ));
                    info.done(Ok(did_read));
                } else {
                    add_line(format_args!(
                        "FED     {:5} bytes, offset {:5}",
                        did_read,
                        info.offset(),
                    ));
                    info.done(Ok(did_read));
                }
            }
        } else {
            // Example only: Use your native filesystem
            // synchronization to wait for more requests
            sleep_ms(10);
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    let worker = thread::spawn(process_queue);

    {
        // Create a System object and initialize.
        let system = fmod::System::new()?;
        system.init(1, fmod::InitFlags::Normal)?;
        system.set_stream_buffer_size(32768, fmod::TimeUnit::RawBytes)?;
        system.set_file_system_async::<MyFileSystem>(2048)?;

        let sound = system.create_stream(
            media!("wave.mp3"),
            fmod::Mode::LoopNormal | fmod::Mode::D2 | fmod::Mode::IgnoreTags,
        )?;

        let channel = system.play_sound(&sound, None, false)?;

        let mut sound = Some(sound);

        // Main loop.
        while !example.btn_press(Buttons::Quit) {
            example.update()?;

            if let Some(sound) = &sound {
                let open_state = sound.get_open_state_info()?;
                if open_state.starving {
                    add_line("Starving");
                }
                channel.set_mute(open_state.starving)?;
            }

            if example.btn_press(Buttons::Action1) {
                sound.take();
                add_line("Released sound");
            }

            system.update()?;

            example.draw("==================================================");
            example.draw("Async IO Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2022.");
            example.draw("==================================================");
            example.draw("");
            example.draw(format_args!(
                "Press {} to release playing stream",
                Buttons::Action1.name(),
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");
            draw_lines(&mut example);

            sleep_ms(15);
        }
    }

    THREAD_QUIT.store(true, SeqCst);
    worker.join().unwrap();

    example.close()?;

    Ok(())
}
