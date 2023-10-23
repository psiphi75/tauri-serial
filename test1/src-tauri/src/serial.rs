use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use serial2::{self, SerialPort};

pub type Callback = Arc<Mutex<dyn FnMut(&[u8]) + Send + Sync>>;

#[derive(Debug)]
pub struct Serial {
    is_open: Mutex<bool>,
    sender: mpsc::Sender<Vec<u8>>,
}

impl Serial {
    pub fn list_ports() -> Vec<String> {
        let mut result = vec![];

        match serial2::SerialPort::available_ports() {
            Err(e) => {
                eprintln!("Failed to enumerate serial ports: {}", e);
            }
            Ok(ports) => {
                for port in ports {
                    let p: String = format!("{}", port.display());
                    result.push(p);
                }
            }
        }
        result
    }

    pub fn open(port: &str, baud: u32, read_timeout_ms: u64, read_cb: Callback) -> Self {
        // Open the port
        let mut serial = SerialPort::open(port, baud).expect("Unable to open port");
        if read_timeout_ms > 0 {
            let timeout = std::time::Duration::from_millis(read_timeout_ms);
            serial
                .set_read_timeout(timeout)
                .expect("Unable to set read timeout");
        }

        let (sender, receiver) = mpsc::channel::<Vec<u8>>();
        let is_open: Mutex<bool> = true.into();

        // Create the comms channels

        // Start the thread
        let mut buf = [0; 1024];
        std::thread::spawn(move || loop {
            // Read data and send it back via the callback
            let read = serial.read(&mut buf).expect("Unable to read from port");
            if read > 0 {
                let cb = &mut read_cb.lock().unwrap();
                cb(&buf[0..read]);
            }

            // Write any data we received
            {
                let recv_data = receiver.try_recv();
                if !recv_data.is_err() {
                    let data = recv_data.unwrap();
                    serial.write(&data).expect("Unable to write to serial port");
                }
            }

            // Close the thread if we have to.
            {
                let remain_open = is_open.lock().unwrap();
                if !*remain_open {
                    break;
                }
            }
        });

        Self {
            is_open: true.into(),
            sender,
        }
    }

    pub fn write(&self, buf: Vec<u8>) {
        {
            let is_open = self.is_open.lock().unwrap();
            if !*is_open {
                return;
            }
        }

        self.sender.send(buf).unwrap();
    }

    pub fn close(&self) {
        let mut is_open = self.is_open.lock().unwrap();
        *is_open = false;
    }
}
