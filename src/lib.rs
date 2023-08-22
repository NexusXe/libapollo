#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

#![allow(incomplete_features)]
#![feature(core_intrinsics)]
#![feature(const_size_of_val)]
#![feature(const_likely)]

use parameters::*;
pub mod parameters;
pub mod telemetry;

pub static mut LATITUDE: [u8; LATITUDE_SIZE] = [0u8; LATITUDE_SIZE];
pub static mut LONGITUDE: [u8; LONGITUDE_SIZE] = [0u8; LONGITUDE_SIZE];
pub static mut ALTITUDE: [u8; ALTITUDE_SIZE] = [0u8; ALTITUDE_SIZE];
pub static mut VOLTAGE: [u8; VOLTAGE_SIZE] = [0u8; VOLTAGE_SIZE];
pub static mut TEMPERATURE: [u8; TEMPERATURE_SIZE] = [0u8; TEMPERATURE_SIZE];

pub fn generate_packet(get_location: fn() -> ([u8; LATITUDE_SIZE], [u8; LONGITUDE_SIZE]), get_altitude: fn() -> [u8; ALTITUDE_SIZE], get_voltage: fn() -> [u8; VOLTAGE_SIZE], get_temperature: fn() -> [u8; TEMPERATURE_SIZE]) -> [u8; TOTAL_MESSAGE_LENGTH_BYTES] {
    unsafe { // TODO: horrific
        
        ALTITUDE = get_altitude();
        VOLTAGE = get_voltage();
        TEMPERATURE = get_temperature();
        (LATITUDE, LONGITUDE) = get_location();
        
        // _altitude = 1337.69f32.to_be_bytes();
        // _voltage = 420.69f32.to_be_bytes();
        // _temperature = 420.1337f32.to_be_bytes();
        // _latitude = 69.1337f32.to_be_bytes();
        // _longitude = 69.420f32.to_be_bytes();

        // _latitude = 41.1499498f32.to_be_bytes();
        // _longitude = (-87.2426919f32).to_be_bytes();

        let _blocks: telemetry::BlockStack = telemetry::construct_blocks(&ALTITUDE, &VOLTAGE, &TEMPERATURE, &LATITUDE, &LONGITUDE);
        let _packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = telemetry::construct_packet(_blocks);

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
            
            ALTITUDE = 1337.69f32.to_be_bytes();
            VOLTAGE = 420.69f32.to_be_bytes();
            TEMPERATURE = 420.1337f32.to_be_bytes();
            LATITUDE = 69.1337f32.to_be_bytes();
            LONGITUDE = 69.420f32.to_be_bytes();

            let _blocks = telemetry::construct_blocks(&ALTITUDE, &VOLTAGE, &TEMPERATURE, &LATITUDE, &LONGITUDE);
    
            let _packet = telemetry::construct_packet(_blocks);
            telemetry::encode_packet(&_packet);
        }
    }
}