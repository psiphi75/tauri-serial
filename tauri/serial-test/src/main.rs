use serial2::SerialPort;
// use std::io::Read;
use std::time::Duration;

const WRITE_DATA: &[u8] = &[4, 0, 33, 217, 106];

fn main() {
    let mut port = SerialPort::open("/dev/ttyUSB0", |mut settings: serial2::Settings| {
        settings.set_raw();
        settings.set_baud_rate(115200)?;
        settings.set_char_size(serial2::CharSize::Bits8);
        settings.set_stop_bits(serial2::StopBits::Two);
        settings.set_parity(serial2::Parity::None);
        settings.set_flow_control(serial2::FlowControl::None);
        Ok(settings)
    })
    .expect("Failed to open serial port");
    port.set_read_timeout(Duration::from_millis(100)).unwrap();

    let mut read_buf = [0; 1024];

    port.write_all(WRITE_DATA).expect("Write failed!");

    loop {
        match port.read(&mut read_buf) {
            Ok(0) => (),
            Ok(n) => {
                println!("Read data: {:?}", read_buf)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                println!("Waiting for data");
            }
            Err(e) => {
                eprintln!("Error: Failed to read from port: {:?}", e);
                break;
            }
        };
    }
}
