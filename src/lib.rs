use cp211x_uart::HidUart;

/*
Bus 002 Device 020: ID 10c4:ea80 Silicon Labs CP2110 HID UART Bridge

http://gushh.net/blog/ut61e-protocol/
http://smartypies.com/projects/ut171a-data-reader-on-linux/
http://www.kerrywong.com/2021/04/04/teardown-of-a-uni-t-ut61e-true-rms-multimeter/
https://blog.ja-ke.tech/multimeter/2018/03/09/UT61E-ble.html
https://github.com/4x1md/ut61e_py
https://github.com/gulux/Uni-T-CP2110/
https://github.com/pklaus/ut61e-web
https://github.com/rginda/pycp2110/
https://hackaday.io/project/160036-uni-t-ut61e
https://testmeterpro.com/uni-t-ut61e/
https://unix.stackexchange.com/questions/670636/unable-to-use-usb-dongle-based-on-usb-serial-converter-chip
https://web.archive.org/web/20100328015518/http://www.steffenvogel.de/2009/11/29/uni-trend-ut61e-digital-multimeter
https://www.lenr-forum.com/attachment/23908-ut61e-decoding-pdf/

Serial port settings are 19200bps,
7 data bits,
odd parity,
1 stop bit.
The supplied adapter also requires DTR=1 and RTS=0.
*/

const DATA_LEN: usize = 14;
const CR: u8 = 0x0D;
const LF: u8 = 0x0A;

/// UT61E protocol constants
/// Significant bits in digit bytes
const DIGIT_MASK: u8 = 0b00001111;

/// Percent
/// Byte 7 bit 3
const PERCENT: u8 = 0b00001000;

/// Minus
/// Byte 7 bit 2
const NEG: u8 = 0b00000100;

/// Low battery
/// Byte 7 bit 1
const LOW_BAT: u8 = 0b00000010;

/// OL
/// Byte 7 bit 0
const OL: u8 = 0b00000001;

/// Relative mode
/// Byte 8 bit 1
const DELTA: u8 = 0b00000010;

/// UL
/// Byte 9 bit 3
const UL: u8 = 0b00001000;

/// MAX
/// Byte 9 bit 2
const MAX: u8 = 0b00000100;

/// MIN
/// Byte 9 bit 1
const MIN: u8 = 0b00000010;

/// DC
/// Byte 10 bit 3
const DC: u8 = 0b00001000;

/// AC
/// Byte 10 bit 2
const AC: u8 = 0b00000100;

/// AUTO
/// Byte 10 bit 1
const AUTO: u8 = 0b00000010;

/// Hz
/// Byte 10 bit 0
const HZ: u8 = 0b00000001;

/// Hold
/// Byte 11 bit 1
const HOLD: u8 = 0b00000010;

#[derive(Debug)]
pub struct Message {
    pub percent: bool,
    pub minus: bool,
    pub low_battery: bool,
    /// Indicates either overload, or diode being tested is open or polarity is reversed
    pub ol: bool,
    pub delta: bool,
    /// `true` when frequency:
    ///   - < 2 Hz (22 Hz range)
    ///   - < 20 Hz (220 Hz range)
    ///   - duty cycle < 10%
    pub ul: bool,
    pub max: bool,
    pub min: bool,
    pub dc: bool,
    pub ac: bool,
    pub auto: bool,
    pub hz: bool,
    pub hold: bool,
    pub mode: &'static str,
    pub range: &'static str,
    pub units: &'static str,
    pub val: f64,
    pub norm_val: f64,
    pub norm_units: &'static str,
}

type Range = (&'static str, &'static str, f64);

type RangeTable = [Option<Range>; 8];

const RANGE_V: RangeTable = [
    Some(("2.2000", "V", 0.0001)),
    Some(("22.000", "V", 0.001)),
    Some(("220.00", "V", 0.01)),
    Some(("1000.0", "V", 0.1)),
    Some(("220.00", "mV", 0.01)),
    None,
    None,
    None,
];

const RANGE_R: RangeTable = [
    Some(("220.00", "Ohm", 0.01)),
    Some(("2.2000", "kOhm", 0.0001)),
    Some(("22.000", "kOhm", 0.001)),
    Some(("220.00", "kOhm", 0.01)),
    Some(("2.2000", "MOhm", 0.0001)),
    Some(("22.000", "MOhm", 0.001)),
    Some(("220.00", "MOhm", 0.01)),
    None,
];

const RANGE_C: RangeTable = [
    Some(("22.000", "nF", 0.001)),
    Some(("220.00", "nF", 0.01)),
    Some(("2.2000", "uF", 0.0001)),
    Some(("22.000", "uF", 0.001)),
    Some(("220.00", "uF", 0.01)),
    Some(("2.2000", "mF", 0.0001)),
    Some(("22.000", "mF", 0.001)),
    Some(("220.00", "mF", 0.01)),
];

const RANGE_F: RangeTable = [
    Some(("220.00", "Hz", 0.01)),
    Some(("2200.0", "Hz", 0.1)),
    None,
    Some(("22.000", "kHz", 0.001)),
    Some(("220.00", "kHz", 0.01)),
    Some(("2.2000", "MHz", 0.0001)),
    Some(("22.000", "MHz", 0.001)),
    Some(("220.00", "MHz", 0.01)),
];

const RANGE_I_UA: RangeTable = [
    Some(("220.00", "uA", 0.01)),
    Some(("2200.0", "uA", 0.1)),
    None,
    None,
    None,
    None,
    None,
    None,
];

const RANGE_I_MA: RangeTable = [
    Some(("22.000", "mA", 0.001)),
    Some(("220.00", "mA", 0.01)),
    None,
    None,
    None,
    None,
    None,
    None,
];

const RANGE_I_A: RangeTable = [
    Some(("10.000", "A", 0.001)),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

const RANGE_PERCENT: RangeTable = [
    Some(("100.0", "%", 0.01)),
    Some(("100.0", "%", 0.01)),
    None,
    Some(("100.0", "%", 0.01)),
    Some(("100.0", "%", 0.01)),
    Some(("100.0", "%", 0.01)),
    Some(("100.0", "%", 0.01)),
    None,
];

const MEAS_TYPE: [Option<(&str, Option<&RangeTable>)>; 16] = [
    Some(("A", Some(&RANGE_I_A))),
    Some(("Diode", Some(&RANGE_V))),
    Some(("Hz/%", Some(&RANGE_F))),
    Some(("Ohm", Some(&RANGE_R))),
    Some(("deg", None)),
    Some(("Buzzer", Some(&RANGE_R))),
    Some(("Cap", Some(&RANGE_C))),
    None,
    None,
    Some(("A", Some(&RANGE_I_A))),
    None,
    Some(("V/mV", Some(&RANGE_V))),
    None,
    Some(("uA", Some(&RANGE_I_UA))),
    Some(("ADP", None)),
    Some(("mA", Some(&RANGE_I_MA))),
];

#[derive(Debug, onlyerror::Error)]
pub enum Error {
    /// Message could not be read.
    Read,
    /// Message failed to parse.
    Parse,
    /// Uart error.
    Uart(#[from] cp211x_uart::Error),
}

pub struct UT61E {
    port: HidUart,
}

impl UT61E {
    pub fn new(port: HidUart) -> UT61E {
        UT61E { port }
    }

    pub fn read_message(&mut self) -> Result<Message, Error> {
        let mut raw_data: [u8; 14] = [0; 14];
        self.read_raw_message(&mut raw_data)?;

        let percent = raw_data[7] & PERCENT != 0;
        let minus = raw_data[7] & NEG != 0;
        let low_battery = raw_data[7] & LOW_BAT != 0;
        let ol = raw_data[7] & OL != 0;
        let delta = raw_data[8] & DELTA != 0;
        let ul = raw_data[9] & UL != 0;
        let max = raw_data[9] & MAX != 0;
        let min = raw_data[9] & MIN != 0;
        let dc = raw_data[10] & DC != 0;
        let ac = raw_data[10] & AC != 0;
        let auto = raw_data[10] & AUTO != 0;
        let hz = raw_data[10] & HZ != 0;
        let hold = raw_data[11] & HOLD != 0;

        let meas_type_index = (raw_data[6] & 0x0F) as usize;
        let meas_type = MEAS_TYPE[meas_type_index].ok_or(Error::Parse)?;
        let range_id = (raw_data[0] & 0b00000111) as usize;

        let mode = meas_type.0;
        let (range, units, multiplier) = if percent {
            RANGE_PERCENT[range_id].ok_or(Error::Parse)?
        } else if hz {
            RANGE_F[range_id].ok_or(Error::Parse)?
        } else {
            let range_table = meas_type.1.ok_or(Error::Parse)?;
            range_table[range_id].ok_or(Error::Parse)?
        };

        let mut val = 0f64;
        for n in 1..=5 {
            let digit = (raw_data[n] & DIGIT_MASK) as f64;
            val = val * 10.0 + digit;
        }
        val *= multiplier;
        if minus {
            val = -val;
        }

        let (norm_val, norm_units) = normalize_val(val, units);

        let message = Message {
            percent,
            minus,
            low_battery,
            ol,
            delta,
            ul,
            max,
            min,
            ac,
            dc,
            auto,
            hz,
            hold,
            mode,
            range,
            units,
            val,
            norm_val,
            norm_units,
        };

        Ok(message)
    }

    fn read_exact(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let mut pos = 0;
        while pos < buffer.len() {
            pos += self.port.read(&mut buffer[pos..])?;
        }

        Ok(())
    }

    /// Reads from RX until CRLF is found,
    /// or gives up after a reasonable number of attempts.
    fn read_raw_message(&mut self, data: &mut [u8; DATA_LEN]) -> Result<(), Error> {
        const MAX_RETRIES: usize = 10;

        let mut buf = [0; 1];
        let mut deq: heapless::Deque<u8, 14> = heapless::Deque::new();

        for _ in 0..(MAX_RETRIES * DATA_LEN) {
            self.read_exact(&mut buf)?;

            let prev = deq.back().copied();

            if deq.is_full() {
                deq.pop_front();
            }

            let _ = deq.push_back(buf[0]);

            if buf[0] == LF {
                if prev == Some(CR) && deq.is_full() {
                    let (s0, s1) = deq.as_slices();
                    data[0..s0.len()].copy_from_slice(s0);
                    data[s0.len()..deq.len()].copy_from_slice(s1);
                    return Ok(());
                } else {
                    deq.clear();
                }
            }
        }

        Err(Error::Read)
    }
}

/// Normalizes measured value to standard units. Voltage
/// is normalized to Volt, current to Ampere, resistance to Ohm,
/// capacitance to Farad and frequency to Herz.
/// Other units are not changed.
fn normalize_val(val: f64, units: &'static str) -> (f64, &'static str) {
    let (norm_mult, norm_units) = match units {
        // Voltage
        "V" => (1.0, "V"),
        "mV" => (1E-03, "V"),
        // Current
        "A" => (1.0, "A"),
        "mA" => (1E-03, "A"),
        "uA" => (1E-06, "A"),
        // Resistance
        "Ohm" => (1.0, "Ohm"),
        "kOhm" => (1E03, "Ohm"),
        "MOhm" => (1E06, "Ohm"),
        // Capacitance
        "nF" => (1E-9, "F"),
        "uF" => (1E-6, "F"),
        "mF" => (1E-3, "F"),
        // Frequency
        "Hz" => (1.0, "Hz"),
        "kHz" => (1E03, "Hz"),
        "MHz" => (1E06, "Hz"),
        // Percent
        "%" => (1.0, "%"),
        _ => panic!("unexpected units"),
    };

    return (val * norm_mult, norm_units);
}
