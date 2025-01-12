use clap::Parser;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::time::Duration;

use std::{thread, time};

use evdev::{Device, Key};
use std::path::Path;

/// Headless driver program to control the fireplace
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// MPV IPC control socket
    #[arg(short, long)]
    mpv_socket: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting headless!");

    let args = Args::parse();
    thread::sleep(time::Duration::new(1, 0));
    let mut mpv_socket = UnixStream::connect(args.mpv_socket.unwrap_or("/tmp/mpvsocket".into()))?;
    println!("mpv socket opened.");

    // List available devices
    println!("Listing all available libev devices.");
    for (i, path) in std::fs::read_dir("/dev/input/")?.enumerate() {
        let path = path?.path();
        if path.to_string_lossy().contains("event") {
            if let Ok(device) = Device::open(&path) {
                println!("{}. {:?} - {}", i, path, device.name().unwrap_or("Unknown"));
            }
        }
    }

    // TODO: hardcoded keyboard device here.
    println!("Opening /dev/input/event3");
    let path = Path::new("/dev/input/event3");
    let mut device = Device::open(path)?;

    println!(
        "Starting to monitor: {} ({:?})",
        device.name().unwrap_or("Unknown"),
        path
    );

    let mut heater = HeaterController::new()?;

    // Event loop
    loop {
        for event in device.fetch_events()? {
            // Only process key events
            if event.event_type() == evdev::EventType::KEY {
                let key = Key::new(event.code());
                match event.value() {
                    0 => println!("Key released: {:?}", key),
                    1 => {
                        println!("Key pressed: {:?}", key);
                        mpv_socket.write_all(format!("show-text {:?}/n", key).as_bytes())?;
                        mpv_socket.flush()?;
                        match key {
                            Key::KEY_INSERT => heater.heater_on()?,
                            Key::KEY_DELETE => heater.heater_off()?,
                            _ => (),
                        }
                    }
                    2 => println!("Key repeated: {:?}", key),
                    _ => (),
                }
            }
        }
    }
}

enum HeaterState {
    Off,
    On,
}

struct HeaterController {
    port: Box<dyn serialport::SerialPort>,
    state: HeaterState,
}

impl HeaterController {
    fn new() -> Result<Self, anyhow::Error> {
        let mut port = serialport::new("/dev/ttyACM0", 115_200)
            .timeout(Duration::from_millis(100))
            .open()
            .expect("Failed to open port");
        port.flush()?;
        Ok(HeaterController {
            port,
            state: HeaterState::Off,
        })
    }

    fn heater_off(&mut self) -> Result<(), anyhow::Error> {
        self.port.write_all(b"off\r")?;
        let mut serial_buf: Vec<u8> = vec![0; 64];
        self.port.read(serial_buf.as_mut_slice())?;
        self.state = HeaterState::Off;
        Ok(())
    }

    fn heater_on(&mut self) -> Result<(), anyhow::Error> {
        self.port.write_all(b"on\r")?;
        let mut serial_buf: Vec<u8> = vec![0; 64];
        self.port.read(serial_buf.as_mut_slice())?;
        self.state = HeaterState::On;

        Ok(())
    }
}
