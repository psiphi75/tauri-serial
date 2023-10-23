// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use test1::serial::Serial;

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
    println!("serialopen");
    // window.emit("serial-read", vec![0; 5]).unwrap();
    // Make sure the port exists
    if !Serial::list_ports().contains(&msg.port) {
        return String::from("Port not found");
    }

    // Make sure the inner value is not yet set
    let mut inner_value = state.inner.lock().unwrap();
    if let Some(_) = &(*inner_value) {
        // Serial port is running, don't open it again
        return String::from("Port already open");
    }

    // let window_handle = window.handle();
    let cb: Arc<std::sync::Mutex<dyn FnMut(&[u8]) + Send + Sync>> =
        Arc::new(std::sync::Mutex::new(move |data: &_| {
            window.emit("serial-read", data).unwrap();
        }));
    let serial = Serial::open(&msg.port, msg.baud, msg.read_timout_ms as u64, cb);
    *inner_value = Some(serial);

    String::from("Port opened")
}

#[tauri::command]
fn serial_close(state: tauri::State<'_, SerialComms>) {
    let mut inner_value = state.inner.lock().unwrap();
    if let Some(serial) = &(*inner_value) {
        serial.close();
        *inner_value = None;
    }
}

struct SerialComms {
    inner: std::sync::Mutex<Option<Serial>>,
}

fn main() {
    tauri::Builder::default()
        .manage(SerialComms {
            inner: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            serial_open,
            serial_list_ports,
            serial_close
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
