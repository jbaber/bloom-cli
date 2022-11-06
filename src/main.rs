use xxhash_rust::xxh32;
use xxhash_rust::const_xxh32;

// const SEED: u32 = 0;
// 
// const TEST: u32 = const_xxh32::xxh32(b"TEST", SEED);
// 
// fn test_input(text: &str) -> bool {
//     match xxh32::xxh32(text.as_bytes(), SEED) {
//         TEST => true,
//         _ => false
//     }
// }

/*
*  Approximately optimal m (# bits) and k (# hashes) for e error rate and n (# records):
*  m = -n * log e / (log 2)^2
*  k = -log e
*
*  (logs base 2)
*
*  So for error rate of .01 and n = 500000
*  m = 3321928 bits (fits in 103811 u32s)
*      This seems to be a terrible approximation, since 500000 can fit in only 
*      500000 / 8 = 62500 u32s.
*  k = 7
*
*  Using the xxHash with 7 different seeds as 7 different hashses, try to start your table
*/


fn is_in_filter(bytes: &[u8], filter: &Vec<u32>) -> Result<bool, ()> {
    let mut to_return = true;

    /* Check each hash */
    for hash_num in 0..8 {
        let (whichint, whichbit) = bit_array_indices(xxh32::xxh32(bytes, hash_num));
        if let Ok(is_set) = bit_set(filter[whichint], whichbit) {
            if !is_set {
                to_return = false;
            }
        }
        else {
            return Err(());
        }
    }

    Ok(to_return)
}


fn bit_set(an_int: u32, bit_index: u8) -> Result<bool, String> {
    if bit_index > 31 {
        Err(format!("{} > 31", bit_index))
    }
    else {
        Ok((((an_int & (0x80000000 >> bit_index)) >> (31 - bit_index)) & 0x1) == 1)
    }
}


fn set_bit(an_int: u32, bit_index: u8) -> Result<u32, String> {
    if bit_index <= 31 {
        Ok(an_int | (0x80000000 >> bit_index))
    }
    else {
        Err(format!("{} > 31", bit_index))
    }
}


/// In `filter`, once you've chose the appropriate u32, which particular bit should
/// be flipped?
fn bit_index(i: u32) -> u8 {
    return (i % 32) as u8;
}


/// Which u32 in `filter` should `i` be in?
fn u32_index(i: u32) -> usize {
    (i / 32) as usize
}


fn bit_array_indices(i: u32) -> (usize, u8) {
    return (u32_index(i), bit_index(i));
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_index() {
        /* 0-31 32-63 64-96 ...  */
        for i in 0..32 {
            assert_eq!(u32_index(i), 0);
        }
        for i in 32..64 {
            assert_eq!(u32_index(i), 1);
        }
        for i in 64..96 {
            assert_eq!(u32_index(i), 2);
        }
        for i in 96..128 {
            assert_eq!(u32_index(i), 3);
        }
        for i in 128..160 {
            assert_eq!(u32_index(i), 4);
        }
        for i in 160..192 {
            assert_eq!(u32_index(i), 5);
        }
        for i in 192..224 {
            assert_eq!(u32_index(i), 6);
        }
        for i in 224..256 {
            assert_eq!(u32_index(i), 7);
        }
        for i in 256..288 {
            assert_eq!(u32_index(i), 8);
        }
        for i in 288..320 {
            assert_eq!(u32_index(i), 9);
        }
        for i in 320..352 {
            assert_eq!(u32_index(i), 10);
        }
        for i in 352..384 {
            assert_eq!(u32_index(i), 11);
        }
        for i in 384..416 {
            assert_eq!(u32_index(i), 12);
        }
        for i in 416..448 {
            assert_eq!(u32_index(i), 13);
        }
        for i in 448..480 {
            assert_eq!(u32_index(i), 14);
        }
        for i in 480..512 {
            assert_eq!(u32_index(i), 15);
        }
        for i in 512..544 {
            assert_eq!(u32_index(i), 16);
        }
        for i in 544..576 {
            assert_eq!(u32_index(i), 17);
        }
        for i in 576..608 {
            assert_eq!(u32_index(i), 18);
        }
        for i in 608..640 {
            assert_eq!(u32_index(i), 19);
        }
        for i in 640..672 {
            assert_eq!(u32_index(i), 20);
        }
        for i in 672..704 {
            assert_eq!(u32_index(i), 21);
        }
        assert_eq!(u32_index(500000), 15625);
    }

    #[test]
    fn test_bit_index() {
        for i in 0..32 {
            assert_eq!(bit_index(i), i as u8);
        }
        for i in 32..64 {
            assert_eq!(bit_index(i), (i - 32) as u8);
        }
        for i in 64..96 {
            assert_eq!(bit_index(i), (i - 64) as u8);
        }
    }

    #[test]
    fn test_bit_array_indices() {
        for i in 0..32 {
            assert_eq!(bit_array_indices(i), (0, i as u8));
        }
        for i in 32..64 {
            assert_eq!(bit_array_indices(i), (1, (i - 32) as u8));
        }
        for i in 64..96 {
            assert_eq!(bit_array_indices(i), (2, (i - 64) as u8));
        }
    }


    #[test]
    fn test_bit_set() {
        for i in 0..31 {
            assert_eq!(bit_set(0x00000000, i), Ok(false));
        }
        for i in 32..=0xff {
            assert!(bit_set(0x00000000, i).is_err());
        }
        let deadbeef_bits = vec![0, 1, 3, 4, 5, 6, 8, 10, 12, 13, 15, 16, 18,
                19, 20, 21, 22, 24, 25, 26, 28, 29, 30, 31];
        for i in 0..32 {
            if deadbeef_bits.contains(&i) {
                assert_eq!(bit_set(0xdeadbeef, i), Ok(true));
            }
            else if i < 32 {
                assert_eq!(bit_set(0xdeadbeef, i), Ok(false));
            }
            else {
                assert!(bit_set(0xdeadbeef, i).is_err());
            }
        }
    }

    #[test]
    fn test_set_bit() {
        assert_eq!(set_bit(0x00000000,  0), Ok(0x80000000));
        assert_eq!(set_bit(0x00000000,  1), Ok(0x40000000));
        assert_eq!(set_bit(0x00000000,  2), Ok(0x20000000));
        assert_eq!(set_bit(0x00000000,  3), Ok(0x10000000));
        assert_eq!(set_bit(0x00000000,  4), Ok(0x08000000));
        assert_eq!(set_bit(0x00000000,  5), Ok(0x04000000));
        assert_eq!(set_bit(0x00000000,  6), Ok(0x02000000));
        assert_eq!(set_bit(0x00000000,  7), Ok(0x01000000));
        assert_eq!(set_bit(0x00000000,  8), Ok(0x00800000));
        assert_eq!(set_bit(0x00000000,  9), Ok(0x00400000));
        assert_eq!(set_bit(0x00000000, 10), Ok(0x00200000));
        assert_eq!(set_bit(0x00000000, 11), Ok(0x00100000));
        assert_eq!(set_bit(0x00000000, 12), Ok(0x00080000));
        assert_eq!(set_bit(0x00000000, 13), Ok(0x00040000));
        assert_eq!(set_bit(0x00000000, 14), Ok(0x00020000));
        assert_eq!(set_bit(0x00000000, 15), Ok(0x00010000));
        assert_eq!(set_bit(0x00000000, 16), Ok(0x00008000));
        assert_eq!(set_bit(0x00000000, 17), Ok(0x00004000));
        assert_eq!(set_bit(0x00000000, 18), Ok(0x00002000));
        assert_eq!(set_bit(0x00000000, 19), Ok(0x00001000));
        assert_eq!(set_bit(0x00000000, 20), Ok(0x00000800));
        assert_eq!(set_bit(0x00000000, 21), Ok(0x00000400));
        assert_eq!(set_bit(0x00000000, 22), Ok(0x00000200));
        assert_eq!(set_bit(0x00000000, 23), Ok(0x00000100));
        assert_eq!(set_bit(0x00000000, 24), Ok(0x00000080));
        assert_eq!(set_bit(0x00000000, 25), Ok(0x00000040));
        assert_eq!(set_bit(0x00000000, 26), Ok(0x00000020));
        assert_eq!(set_bit(0x00000000, 27), Ok(0x00000010));
        assert_eq!(set_bit(0x00000000, 28), Ok(0x00000008));
        assert_eq!(set_bit(0x00000000, 29), Ok(0x00000004));
        assert_eq!(set_bit(0x00000000, 30), Ok(0x00000002));
        assert_eq!(set_bit(0x00000000, 31), Ok(0x00000001));
        for i in 32..=0xff {
            assert!(set_bit(0x00000000, i).is_err());
        }

        /*
        *   f     f  e b   f   f      f    f
        *   !     !  ! !   !   !      !    !
        * 1101 1110 1010 1101 1011 1110 1110 1111
        * 0123 4567 8901 2345 6789 0123 4567 8901
        *             11 1111 1111 2222 2222 2233
        *   d    e    a    d    b    e    e    f
        */
        assert_eq!(set_bit(0xdeadbeef,  0), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef,  1), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef,  2), Ok(0xfeadbeef));
        assert_eq!(set_bit(0xdeadbeef,  3), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef,  4), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef,  5), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef,  6), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef,  7), Ok(0xdfadbeef));
        assert_eq!(set_bit(0xdeadbeef,  8), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef,  9), Ok(0xdeedbeef));
        assert_eq!(set_bit(0xdeadbeef, 10), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 11), Ok(0xdebdbeef));
        assert_eq!(set_bit(0xdeadbeef, 12), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 13), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 14), Ok(0xdeafbeef));
        assert_eq!(set_bit(0xdeadbeef, 15), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 16), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 17), Ok(0xdeadfeef));
        assert_eq!(set_bit(0xdeadbeef, 18), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 19), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 20), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 21), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 22), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 23), Ok(0xdeadbfef));
        assert_eq!(set_bit(0xdeadbeef, 24), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 25), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 26), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 27), Ok(0xdeadbeff));
        assert_eq!(set_bit(0xdeadbeef, 28), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 29), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 30), Ok(0xdeadbeef));
        assert_eq!(set_bit(0xdeadbeef, 31), Ok(0xdeadbeef));
        for i in 32..=0xff {
            assert!(set_bit(0xdeadbeef, i).is_err());
        }
    }
}


fn main() {
//  let mut filter: Vec<u32> = vec![0; 103811];
 // let input = "Hey, there";
 // println!("{:?}", xxh32::xxh32(input.as_bytes(), 0));
 // println!("{:?}", xxh32::xxh32(input.as_bytes(), 0));
 // println!("{:?}", xxh32::xxh32(input.as_bytes(), 1));
    // println!("{:?}", test_input("tEST"));
    // println!("{:?}", test_input("TEST"));
}
