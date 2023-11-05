// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_serial::serial::Serial;

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct ListPortsPayload {
    ports: Vec<String>,
}

#[derive(Clone, serde::Deserialize)]
struct OpenSerialMsg {
    port: String,
    baud: u32,
    read_timout_ms: u32,
}

#[tauri::command]
fn serial_list_ports() -> ListPortsPayload {
    ListPortsPayload {
        ports: Serial::list_ports(),
    }
}

#[tauri::command]
fn serial_open(
    window: tauri::Window,
    msg: OpenSerialMsg,
    state: tauri::State<'_, SerialComms>,
) -> String {
    println!("opening port: {}", msg.port);
    // Make sure the port exists
    if !Serial::list_ports().contains(&msg.port) {
        return String::from("Port not found");
    }

    // Make sure the inner value is not yet set
    let mut inner_value = state.lock().unwrap();
    if (*inner_value).is_some() {
        // Serial port is running, don't open it again
        return String::from("Port already open");
    }

    let cb = Arc::new(Mutex::new(move |data: Vec<u8>| {
        window.emit("serial-read", data).unwrap();
    }));
    let serial = Serial::open(&msg.port, msg.baud, msg.read_timout_ms as u64, cb);
    *inner_value = Some(serial);

    String::from("Port opened")
}

#[tauri::command]
fn serial_close(state: tauri::State<'_, SerialComms>) {
    let mut inner_value = state.lock().unwrap();
    if let Some(serial) = &(*inner_value) {
        serial.close();
        *inner_value = None;
    }
}

fn to_array(s: &str) -> Vec<u8> {
    let parts = s[1..s.len()-1].split(",");
    let mut result = vec![];
    for part in parts {
        println!("part: {}", part);
        result.push(part.parse().unwrap());
    }
    result
}

type SerialComms = Arc<Mutex<Option<Serial>>>;

fn main() {
    let serial_comms: SerialComms = Arc::new(Mutex::new(None));
    let serial_comms2: SerialComms = serial_comms.clone();
    tauri::Builder::default()
        .manage(serial_comms)
        .setup(move |app| {
            app.listen_global("serial-write", move |event| {
                println!("Got 'serial-write' event payload: {:?}", event.payload());
                let serial_c = serial_comms2.lock().unwrap();
                if let Some(serial) = &(*serial_c) {
                    if let Some(data) = event.payload() {
                        // let bytes = data.to_owned().as_bytes().to_vec();
                        let bytes = to_array(data);
                        println!("bytes: {:?} -> {:?}", bytes, data);
                        serial.write(bytes);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            serial_list_ports,
            serial_open,
            serial_close
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
