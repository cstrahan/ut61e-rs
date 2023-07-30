use std::time::Duration;

use cp211x_uart::{DataBits, FlowControl, HidUart, Parity, StopBits, UartConfig};
use ut61e::UT61E;

// thread 'main' panicked at 'range end index 15 out of range for slice of length 14', /home/cstrahan/src/cp211x_uart/src/lib.rs:336:21

fn run() -> Result<(), cp211x_uart::Error> {
    let manager = hidapi::HidApi::new()?;
    let device_info = manager
        .device_list()
        .filter(|device_info| {
            device_info.vendor_id() == 0x10C4 && device_info.product_id() == 0xEA80
        })
        .next();

    if let Some(device_info) = device_info {
        let device = device_info.open_device(&manager)?;

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

        // uart.write(&[0x01, 0x02, 0x03][..]).unwrap();
        // let mut buf: [u8; 256] = [0; 256];
        // uart.read(&mut buf).unwrap();

        let mut ut61e = UT61E::new(uart);

        loop {
            let message = ut61e.read_message();
            println!("{message:?}\n\n");

            // std::thread::sleep(Duration::from_secs(1));
        }
    } else {
        println!("no device detected");
        Ok(())
    }
}

fn main() {
    run().unwrap()
}
