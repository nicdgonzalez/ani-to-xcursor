#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    _ = ani::de::Ani::from_bytes(data);
});
