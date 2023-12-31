extern crate std;

use super::*;

#[test]
fn make_packet() {
    let _status = 411240910u32.to_be_bytes();
    let _altitude = 1337.69f32.to_be_bytes();
    let _voltage = 420.69f32.to_be_bytes();
    let _temperature = 420.1337f32.to_be_bytes();
    let _latitude = 69.1337f32.to_be_bytes();
    let _longitude = 69.420f32.to_be_bytes();

    let _blocks = telemetry::construct_blocks(&[
        _status,
        _altitude,
        _voltage,
        _temperature,
        _latitude,
        _longitude,
    ]);

    let _packet = telemetry::construct_packet(_blocks);
    telemetry::encode_packet(&_packet);
}
