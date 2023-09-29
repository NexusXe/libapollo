/// Packs bools into a u8, where the first bit in the input is the least-significant
/// bit in the output u8. For example, a bool array of `[TFFFFFTF]` would result in a
/// u8 whose binary representation (LE) would be `0b01000001`.
///
/// TODO: Ensure portability, especially when the transmitter and receiver differ in
/// endianness.
pub const fn pack_bools_to_byte<const S: usize>(bools: [bool; S]) -> u8 {
    let mut i: usize = 0;
    let mut packed_bools: u8 = 0b00000000;
    while i < bools.len() {
        packed_bools |= (bools[i] as u8) << i;
        i += 1;
    }
    packed_bools
}

/// Unpacks a u8 into a [StatusBoolsArray], rather quickly.
/// Note that this does not preserve the number of bools
/// that we began with.
///
/// Some implementation info:
///
/// unpacked bools will always be either 0x00 or 0x01*, so we can
/// tell the compiler that it doesn't have to convert to a bool
/// (which it would do with a `cmp` to 0). this is bit-identical
/// to `bools[i] = std::mem::transmute(x);` but more code-safe
///
/// *technically, rust checks the value of a bool by only checking
/// its least-significant bit; therefore, a bool with the value
/// `0b1110` is `false`, whereas `0b1111` is `true`. however, the
/// compiler is not fond of this and will lead to `SIGILL`s,
/// i think because the compiler will put unaligned values into
/// the "excess" space of the bool, yet the computer is still
/// writing the "excess" bits of the non-standard bool
/// this causes problems, as one may expect, and is technically
/// a memory access violation (fun!)
pub const fn unpack_bools(packed_bools: u8) -> StatusBoolsArray {
    let mut i: usize = 0;
    let mut bools: StatusBoolsArray = [false; 8];
    const MASK_SET: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

    while i < 8 {
        let x = (packed_bools & MASK_SET[i]) >> i;
        if (x != 0) & (x != 1) {
            // using the proper logical and can cause a SIGILL..?
            unreachable!();
        }
        bools[i] = x != 0;
        i += 1;
    }
    bools
}

pub type StatusBoolsArray = [bool; 8];

pub struct StatusFlagsLat {
    lat_sign: bool,
    long_sign: bool,
    voltage_sign: bool,
    gps_lock: bool,
    altitude_regime: [bool; 4],
}

impl Into<u8> for StatusFlagsLat {
    fn into(self) -> u8 {
        0u8 | pack_bools_to_byte([
            self.lat_sign,
            self.long_sign,
            self.voltage_sign,
            self.gps_lock,
            self.altitude_regime[0],
            self.altitude_regime[1],
            self.altitude_regime[2],
            self.altitude_regime[3],
        ])
    }
}

impl StatusFlagsLat {
    pub const fn new(
        _lat_sign: bool,
        _long_sign: bool,
        _voltage_sign: bool,
        _gps_lock: bool,
        _altitude: u16,
    ) -> StatusFlagsLat {
        let _converted_altitude: u8 = {
            let intermediate: u8 = (_altitude / 2000u16) as u8;
            if intermediate > 15 {
                15u8
            } else {
                intermediate
            }
        };

        debug_assert!(_converted_altitude < 16);
        let altitude_bools = unpack_bools(_converted_altitude);
        Self {
            lat_sign: _lat_sign,
            long_sign: _long_sign,
            voltage_sign: _voltage_sign,
            gps_lock: _gps_lock,
            altitude_regime: [
                altitude_bools[0],
                altitude_bools[1],
                altitude_bools[2],
                altitude_bools[3],
            ],
        }
    }

    pub const fn into_byte(self) -> u8 {
        0u8 | pack_bools_to_byte([
            self.lat_sign,
            self.long_sign,
            self.voltage_sign,
            self.gps_lock,
            self.altitude_regime[0],
            self.altitude_regime[1],
            self.altitude_regime[2],
            self.altitude_regime[3],
        ])
    }
}

#[cfg(test)]
mod tests {
    const EXAMPLE_STATUSES: [StatusBoolsArray; 2] = [
        [true, true, false, true, true, true, true, false],
        [false, true, false, false, false, false, false, false],
    ];

    use super::*;

    const fn make_statuses(status_bools: [StatusBoolsArray; 2]) -> [u8; 2] {
        [
            pack_bools_to_byte(status_bools[0]),
            pack_bools_to_byte(status_bools[1]),
        ]
    }

    #[test]
    fn check_status_packing() {
        const ATTEMPT: [u8; 2] = make_statuses(EXAMPLE_STATUSES);
        const EXPECTED_OUTPUT: [u8; 2] = [0b01111011u8, 0b00000010u8];
        assert_eq!(
            ATTEMPT, EXPECTED_OUTPUT,
            "\nexpected: [{:08b}, {:08b}]\nfound:    [{:08b}, {:08b}]",
            EXPECTED_OUTPUT[0], EXPECTED_OUTPUT[1], ATTEMPT[0], ATTEMPT[1]
        );
    }
}
