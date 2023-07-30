use std::time::Duration;

use cp211x_uart::{DataBits, FlowControl, HidUart, Parity, StopBits, UartConfig};

fn log_measurements(device: hidapi::HidDevice) -> Result<(), ut61e::Error> {
    let mut uart = HidUart::new(device).unwrap();

    let config = UartConfig {
        baud_rate: 19200,
        data_bits: DataBits::Bits7,
        stop_bits: StopBits::Short,
        parity: Parity::Odd,
        flow_control: FlowControl::None,
    };

    uart.set_config(&config).unwrap();
    uart.set_read_timeout(Duration::from_millis(50));
    uart.set_write_timeout(Duration::from_millis(500));
    uart.flush_fifos(true, true).unwrap();

    let mut stream = ut61e::Stream::new();

    loop {
        let Some(ch) = read_char(&mut uart).expect("couldn't read char") else { continue; };
        let Some(raw_message) = stream.push(ch) else { continue };
        let message = ut61e::parse_message(&raw_message)?;
        println!("{message:#?}\n\n");
    }
}

fn read_char(port: &mut cp211x_uart::HidUart) -> Result<Option<u8>, cp211x_uart::Error> {
    let mut buf = [0; 1];
    let n = port.read(&mut buf)?;
    if n == 0 {
        Ok(None)
    } else {
        Ok(Some(buf[0]))
    }
}

fn main() {
    let manager = hidapi::HidApi::new().expect("couldn't init hidapi");
    let device_info = manager
        .device_list()
        .filter(|device_info| {
            device_info.vendor_id() == 0x10C4 && device_info.product_id() == 0xEA80
        })
        .next()
        .expect("couldn't find device");

    let device = device_info
        .open_device(&manager)
        .expect("couldn't open device");

    log_measurements(device).unwrap()
}
