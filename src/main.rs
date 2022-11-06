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


/// In `filter`, once you've chose the appropriate u32, which particular bit should
/// be flipped?
fn bit_index(i: u32) -> u8 {
    return (i % 32) as u8;
}


/// Which u32 in `filter` should `i` be in?
fn u32_index(i: u32) -> u32 {
    i / 32
}


fn bit_array_indices(i: u32) -> (u32, u8) {
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
