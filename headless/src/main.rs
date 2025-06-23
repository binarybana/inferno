use clap::Parser;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::time::Duration;
use std::process::{Command, Child};

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
    
    // Start MPV video playback on both monitors
    let socket_path = args.mpv_socket.unwrap_or("/tmp/mpvsocket".into());
    let mut vid_player = VidPlayerState::new(&socket_path);
    vid_player.start_playback()?;
    println!("MPV video playback started on both monitors");
    
    // Wait a moment for MPV to start and create the sockets
    thread::sleep(time::Duration::new(2, 0));
    vid_player.connect_sockets()?;

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
                        vid_player.send_message(&format!("show-text {:?}/n", key))?;
                        match key {
                            Key::KEY_INSERT => heater.heater_on()?,
                            Key::KEY_DELETE => heater.heater_off()?,
                            Key::KEY_ESC => {
                                println!("Escape pressed - shutting down");
                                vid_player.shutdown()?;
                                return Ok(());
                            }
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

struct VidPlayerState {
    mpv_monitor0: Option<Child>,
    mpv_monitor1: Option<Child>,
    socket_path_0: String,
    socket_path_1: String,
    socket_0: Option<UnixStream>,
    socket_1: Option<UnixStream>,
}

impl VidPlayerState {
    fn new(base_socket_path: &str) -> Self {
        Self {
            mpv_monitor0: None,
            mpv_monitor1: None,
            socket_path_0: format!("{}_0", base_socket_path),
            socket_path_1: format!("{}_1", base_socket_path),
            socket_0: None,
            socket_1: None,
        }
    }

    fn start_playback(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home/jason".to_string());
        let video_path = format!("{}/fire_videos/*.webm", home_dir);
        
        println!("Starting MPV instances with video path: {}", video_path);
        
        // Start MPV for monitor 0
        let child0 = Command::new("mpv")
            .arg("--fs")
            .arg("--fs-screen=0")
            .arg(format!("--input-ipc-server={}", self.socket_path_0))
            .arg("--loop-file")
            .arg(&video_path)
            .spawn()?;
        
        // Start MPV for monitor 1
        let child1 = Command::new("mpv")
            .arg("--fs")
            .arg("--fs-screen=1")
            .arg(format!("--input-ipc-server={}", self.socket_path_1))
            .arg("--loop-file")
            .arg(&video_path)
            .spawn()?;
        
        self.mpv_monitor0 = Some(child0);
        self.mpv_monitor1 = Some(child1);
        
        Ok(())
    }

    fn connect_sockets(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.socket_0 = Some(UnixStream::connect(&self.socket_path_0)?);
        self.socket_1 = Some(UnixStream::connect(&self.socket_path_1)?);
        println!("Connected to both MPV sockets");
        Ok(())
    }

    fn send_message(&mut self, msg: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut socket) = self.socket_0 {
            socket.write_all(msg.as_bytes())?;
            socket.flush()?;
        }
        if let Some(ref mut socket) = self.socket_1 {
            socket.write_all(msg.as_bytes())?;
            socket.flush()?;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut process) = self.mpv_monitor0.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        if let Some(mut process) = self.mpv_monitor1.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        self.socket_0 = None;
        self.socket_1 = None;
        println!("Both MPV instances terminated");
        Ok(())
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
