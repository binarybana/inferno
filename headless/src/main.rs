use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
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

fn main() -> Result<()> {
    color_eyre::install().wrap_err("Failed to install color-eyre error handler")?;
    println!("Starting headless!");

    let args = Args::parse();
    
    // Start MPV video playback on both monitors
    let socket_path = args.mpv_socket.unwrap_or("/tmp/mpvsocket".into());
    let mut vid_player = VidPlayerState::new(&socket_path);
    vid_player.start_playback().wrap_err("Failed to start MPV video playback")?;
    println!("MPV video playback started on both monitors");
    
    // Wait a moment for MPV to start and create the sockets
    thread::sleep(time::Duration::new(2, 0));
    vid_player.connect_sockets().wrap_err("Failed to connect to MPV sockets")?;

    // List available devices
    println!("Listing all available libev devices.");
    for (i, path) in std::fs::read_dir("/dev/input/").wrap_err("Failed to read /dev/input/ directory")?.enumerate() {
        let path = path.wrap_err("Failed to read directory entry")?.path();
        if path.to_string_lossy().contains("event") {
            if let Ok(device) = Device::open(&path) {
                println!("{}. {:?} - {}", i, path, device.name().unwrap_or("Unknown"));
            }
        }
    }

    // TODO: hardcoded keyboard device here.
    println!("Opening /dev/input/event3");
    let path = Path::new("/dev/input/event3");
    let mut device = Device::open(path).wrap_err_with(|| format!("Failed to open input device: {:?}", path))?;

    println!(
        "Starting to monitor: {} ({:?})",
        device.name().unwrap_or("Unknown"),
        path
    );

    let mut heater = HeaterController::new().wrap_err("Failed to initialize heater controller")?;

    // Event loop
    loop {
        for event in device.fetch_events().wrap_err("Failed to fetch input events")? {
            // Only process key events
            if event.event_type() == evdev::EventType::KEY {
                let key = Key::new(event.code());
                match event.value() {
                    0 => println!("Key released: {:?}", key),
                    1 => {
                        println!("Key pressed: {:?}", key);
                        vid_player.send_message(&format!("show-text {:?}/n", key)).wrap_err_with(|| format!("Failed to send message for key {:?}", key))?;
                        match key {
                            Key::KEY_INSERT => heater.heater_on().wrap_err("Failed to turn heater on")?,
                            Key::KEY_DELETE => heater.heater_off().wrap_err("Failed to turn heater off")?,
                            Key::KEY_ESC => {
                                println!("Escape pressed - shutting down");
                                vid_player.shutdown().wrap_err("Failed to shutdown video player")?;
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

    fn start_playback(&mut self) -> Result<()> {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home/jason".to_string());
        let video_path = format!("{}/fire_videos/video2.webm", home_dir);
        
        println!("Starting MPV instances with video path: {}", video_path);
        
        // Start MPV for monitor 0
        let child0 = Command::new("mpv")
            .arg("--fs")
            .arg("--fs-screen=0")
            .arg(format!("--input-ipc-server={}", self.socket_path_0))
            .arg("--loop-file")
            .arg(&video_path)
            .spawn()
            .wrap_err("Failed to spawn MPV process for monitor 0")?;
        
        // Start MPV for monitor 1
        let child1 = Command::new("mpv")
            .arg("--fs")
            .arg("--fs-screen=1")
            .arg(format!("--input-ipc-server={}", self.socket_path_1))
            .arg("--loop-file")
            .arg(&video_path)
            .spawn()
            .wrap_err("Failed to spawn MPV process for monitor 1")?;
        
        self.mpv_monitor0 = Some(child0);
        self.mpv_monitor1 = Some(child1);
        
        Ok(())
    }

    fn connect_sockets(&mut self) -> Result<()> {
        self.socket_0 = Some(UnixStream::connect(&self.socket_path_0)
            .wrap_err_with(|| format!("Failed to connect to MPV socket: {}", self.socket_path_0))?);
        self.socket_1 = Some(UnixStream::connect(&self.socket_path_1)
            .wrap_err_with(|| format!("Failed to connect to MPV socket: {}", self.socket_path_1))?);
        println!("Connected to both MPV sockets");
        Ok(())
    }

    fn send_message(&mut self, msg: &str) -> Result<()> {
        if let Some(ref mut socket) = self.socket_0 {
            socket.write_all(msg.as_bytes())
                .wrap_err("Failed to write message to MPV socket 0")?;
            socket.flush()
                .wrap_err("Failed to flush MPV socket 0")?;
        }
        if let Some(ref mut socket) = self.socket_1 {
            socket.write_all(msg.as_bytes())
                .wrap_err("Failed to write message to MPV socket 1")?;
            socket.flush()
                .wrap_err("Failed to flush MPV socket 1")?;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
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
    fn new() -> Result<Self> {
        let mut port = serialport::new("/dev/ttyACM0", 115_200)
            .timeout(Duration::from_millis(100))
            .open()
            .wrap_err("Failed to open serial port /dev/ttyACM0")?;
        port.flush()
            .wrap_err("Failed to flush serial port")?;
        Ok(HeaterController {
            port,
            state: HeaterState::Off,
        })
    }

    fn heater_off(&mut self) -> Result<()> {
        self.port.write_all(b"off\r")
            .wrap_err("Failed to write 'off' command to serial port")?;
        let mut serial_buf: Vec<u8> = vec![0; 64];
        self.port.read(serial_buf.as_mut_slice())
            .wrap_err("Failed to read response from serial port after 'off' command")?;
        self.state = HeaterState::Off;
        Ok(())
    }

    fn heater_on(&mut self) -> Result<()> {
        self.port.write_all(b"on\r")
            .wrap_err("Failed to write 'on' command to serial port")?;
        let mut serial_buf: Vec<u8> = vec![0; 64];
        self.port.read(serial_buf.as_mut_slice())
            .wrap_err("Failed to read response from serial port after 'on' command")?;
        self.state = HeaterState::On;

        Ok(())
    }
}
