#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]

#![allow(incomplete_features)]
#![feature(core_intrinsics)]
#![feature(const_size_of_val)]
#![feature(const_likely)]

use parameters::*;
pub mod parameters;
pub mod telemetry;

pub static mut _latitude: [u8; LATITUDE_SIZE] = [0u8; LATITUDE_SIZE];
pub static mut _longitude: [u8; LONGITUDE_SIZE] = [0u8; LONGITUDE_SIZE];
pub static mut _altitude: [u8; ALTITUDE_SIZE] = [0u8; ALTITUDE_SIZE];
pub static mut _voltage: [u8; VOLTAGE_SIZE] = [0u8; VOLTAGE_SIZE];
pub static mut _temperature: [u8; TEMPERATURE_SIZE] = [0u8; TEMPERATURE_SIZE];

pub fn generate_packet(get_location: fn() -> ([u8; LATITUDE_SIZE], [u8; LONGITUDE_SIZE]), get_altitude: fn() -> [u8; ALTITUDE_SIZE], get_voltage: fn() -> [u8; VOLTAGE_SIZE], get_temperature: fn() -> [u8; TEMPERATURE_SIZE]) -> [u8; TOTAL_MESSAGE_LENGTH_BYTES] {
    unsafe { // TODO: horrific
        
        _altitude = get_altitude();
        _voltage = get_voltage();
        _temperature = get_temperature();
        (_latitude, _longitude) = get_location();
        
        // _altitude = 1337.69f32.to_be_bytes();
        // _voltage = 420.69f32.to_be_bytes();
        // _temperature = 420.1337f32.to_be_bytes();
        // _latitude = 69.1337f32.to_be_bytes();
        // _longitude = 69.420f32.to_be_bytes();

        // _latitude = 41.1499498f32.to_be_bytes();
        // _longitude = (-87.2426919f32).to_be_bytes();

        let _blocks = telemetry::construct_blocks(&_altitude, &_voltage, &_temperature, &_latitude, &_longitude);
    

        let _packet = telemetry::construct_packet(_blocks);

        telemetry::encode_packet(&_packet)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn make_packet() {
        unsafe {
            
            _altitude = 1337.69f32.to_be_bytes();
            _voltage = 420.69f32.to_be_bytes();
            _temperature = 420.1337f32.to_be_bytes();
            _latitude = 69.1337f32.to_be_bytes();
            _longitude = 69.420f32.to_be_bytes();

            let _blocks = telemetry::construct_blocks(&_altitude, &_voltage, &_temperature, &_latitude, &_longitude);
    
            let _packet = telemetry::construct_packet(_blocks);
            telemetry::encode_packet(&_packet);
        }
    }
}