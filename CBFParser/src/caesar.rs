// Reverse engineered code based on Mercedes' c32s.dll

fn get_high(x: u64) -> u32 {
    ((x & 0xFFFFFFFF00000000) >> 32) as u32
}

fn get_low(x: u64) -> u32 {
    (x & 0x00000000FFFFFFFF) as u32
}


pub fn crc_32_accumulate(start_index: u64, a2: u32) -> i32 {
    let mut index: u64 = start_index;
    let mut v2 = a2;
    
    loop {
        v2 -= 1;
        let mut v4 = v2;
        if v4 != 0 {
            break
        }
        index += get_high(index) as u64;
    }
    return index as i32
}