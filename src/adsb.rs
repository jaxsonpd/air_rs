pub fn check_preamble(buf: Vec<u32>) -> Option<(u32, i32, i32)> {
    assert!(buf.len() == 16);

    // Adsb pre amble has the following form:
    //
    // +   -   +   -   -   -   -   +   -   +   -   -   -   -   -   -
    // 0  0.5  1  1.5  2  2.5  3  3.5  4  4.5  5  5.5  6  6.5  7  7.5
    // 0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15 
    let lows = [1, 3, 4, 5, 6, 8, 10, 11, 12, 13, 14, 15];
    let highs = [0, 2, 7, 9];
    let mut min = 800000;

    for high in highs.iter() {
        for low in lows.iter() {
            if buf[*high as usize] < buf[*low as usize] {
                return None;
            }
        }
        if buf[*high as usize] < min {
            min = buf[*high as usize];
        }
    }


    Some(((min as f32 * 0.9) as u32, 0, 0))
}

pub fn check_df(buf: Vec<u32>) -> bool {
    // The preamble is followed by DF which needs to be 17 for adsb
    //
    // This translates too:
    //   1       0       0       0       1
    // +   -   -   +   -   +   -   +   +   -
    // 0  0.5  1  1.5  2  2.5  3  3.5  4  4.5
    // 0   1   2   3   4   5   6   7   8   9 
    let lows = [1, 2, 4, 6, 9];
    let highs = [0, 3, 5, 7, 8];

    for high in highs.iter() {
        for low in lows.iter() {
            if buf[*high as usize] < buf[*low as usize] {
                return false;
            }
        }
    }

    true
}

/// Extract the contents of a packet
/// 
/// buf - the data buffer
/// high - the high level threshold
pub fn extract_packet(buf: Vec<u32>, high: u32) -> Option<Vec<u8>> {
    let mut result: Vec<u8> = Vec::new();
    let mut inter: u8 = 0;
    let mut errors: u8 = 0;

    for i in (0..112).step_by(2) {
        if errors > 2 {
            return None;
        }

        if i % 16 == 0 && i != 0{
            result.push(inter);
            inter = 0;
            errors = 0;
            print!(" ");
            continue;
        }

        if buf[i] > high && buf[i+1] < high { // 1
            inter |= 1 << (7 - ((i)/2 % 8));
        } else if buf[i] < high && buf[i+1] > high {
            inter &= !(1 << (7 - ((i)/2 % 8)));
        } else {
            errors += 1;
        }

        if buf[i] > high {
            print!("1");
        } else {
            print!("0");
        }

        if buf[i+1] > high {
            print!("1");
        } else {
            print!("0");
        }
    }
    print!("\n");

    Some(result)
}

pub fn print_raw_packet(packet: Vec<u8>) {
    for byte in packet.iter() {
        print!("{:08b} ", byte);
    }
    print!("\n");

    for byte in packet.iter() {
        print!("   {:02x}    ", byte);
    }
    print!("\n");
}