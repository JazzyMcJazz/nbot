use std::io::{self, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Spinner {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
    message_length: usize,
}

impl Spinner {
    pub fn new() -> Spinner {
        Spinner {
            handle: None,
            running: Arc::new(AtomicBool::new(false)),
            message_length: 0,
        }
    }
    pub fn start(&mut self, message: String) {
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let spinner_chars = ['|', '/', '-', '\\'];
        let delay = Duration::from_millis(100);

        let moved = message.clone();
        let handle = thread::spawn(move || {
            let mut i = 0;
            while running.load(Ordering::SeqCst) {
                print!("\r{}{}", moved, spinner_chars[i]);
                io::stdout().flush().unwrap();
                i = (i + 1) % spinner_chars.len();
                thread::sleep(delay);
            }
            // Clear the spinner character when stopping
            print!("\r ");
            io::stdout().flush().unwrap();
        });

        self.message_length = message.len() + 1;
        self.handle = Some(handle);
    }

    pub fn stop(&mut self, message: String) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
        print!("\r{:<width$}", " ", width = self.message_length);
        println!("\r{}", message);
        self.message_length = 0;
    }
}
