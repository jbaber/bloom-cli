use argh::FromArgs;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process;
use xxhash_rust::xxh32;

macro_rules! vprintln {
    ($level:expr, $($message:expr),+) => {
        if $level {
            eprintln!($($message ,)+);
        }
    };
}

/* Create an empty bloom filter 3321928 bits long
*
*  Approximately optimal m (# bits) and k (# hashes) for e error rate
*  and n (# records):
*
*  m = -n * log e / (log 2)^2
*  k = -log e
*
*  (logs base 2)
*
*  So for error rate of .01 and n = 500000
*   
*  m = 3321928 bits (fits in 51905 u64s)
*  k = 7
*/
// TODO Somehow make these NonZeroUsize without running into needing to .into()
// later since that's not allowed for const
const M: usize = 3321928;
const FILTER_NUM_BITS: usize = 51906;

static M_NZ: NonZeroUsize = match NonZeroUsize::new(M) {
    Some(good_num) => {
        good_num
    }
    None => {
        panic!("Internal error with non-zero number");
    }
};

fn read_filter_from_disk(filename: &str) -> Result<Vec<u64>, String> {
    let mut file = match File::open(filename) {
        Ok(file) => BufReader::new(file),
        Err(err) => {
            return Err(format!("{}", err));
        },
    };

    let mut to_return = Vec::<u64>::new();
    let mut buffer = [0; 8];

    loop {
        match file.read(&mut buffer[..]) {
            Ok(num_bytes_read) => {
                if num_bytes_read < 8 {
                    return Ok(to_return);
                }
                else {
                    to_return.push(
                        ((buffer[0] as u64) << 56) |
                        ((buffer[1] as u64) << 48) |
                        ((buffer[2] as u64) << 40) |
                        ((buffer[3] as u64) << 32) |
                        ((buffer[4] as u64) << 24) |
                        ((buffer[5] as u64) << 16) |
                        ((buffer[6] as u64) <<  8) |
                        ((buffer[7] as u64) <<  0)
                    );
                }
            },
            Err(err) => {
                return Err(format!("{}", err));
            },
        }
    }
}


fn write_filter_to_disk(filename: &str, filter: &[u64]) -> Result<(), String> {
    let mut file = match File::create(filename) {
        Ok(file) => BufWriter::new(file),
        Err(err) => {
            return Err(err.to_string());
        },
    };

    for int in filter.iter() {
        if file.write_all(&int.to_be_bytes()).is_err() {
            return Err(format!("Failed to write to {}", filename));
        }
    }


    Ok(())
}


fn is_in_filter(bytes: &[u8], filter: &[u64], m: NonZeroUsize) -> Result<bool, String> {
    let mut to_return = true;

    /* Check each hash */
    for hash_num in 0..8 {
        let (whichint, whichbit) = bit_array_indices(
            xxh32::xxh32(bytes, hash_num) as u64,
            m
        );
        if let Ok(is_set) = bit_set(filter[whichint], whichbit) {
            if !is_set {
                to_return = false;
            }
        }
        else {
            return Err("Unable to set bit in filter".to_owned());
        }
    }

    Ok(to_return)
}


fn filter_insert(bytes: &[u8], filter: &mut [u64], m: NonZeroUsize) -> Result<(), String> {
    for hash_num in 0..8 {
        let hash:u32 = xxh32::xxh32(bytes, hash_num);
        let (whichint, whichbit) = bit_array_indices(hash as u64, m);
        match set_bit(filter[whichint], whichbit) {
            Ok(newint) => {filter[whichint] = newint;},
            Err(err) => {return Err(err)},
        }
    }
    Ok(())
}


/// Is bit at index `bit_index` in `an_int` set?
fn bit_set(an_int: u64, bit_index: u8) -> Result<bool, String> {
    if bit_index > 63 {
        Err(format!("{} > 63", bit_index))
    }
    else {
        Ok((((an_int & (0x8000000000000000 >> bit_index)) >> (63 - bit_index)) & 0x1) == 1)
    }
}


/// Return result of setting bit at index `bit_index` in `an_int`
fn set_bit(an_int: u64, bit_index: u8) -> Result<u64, String> {
    if bit_index <= 63 {
        Ok(an_int | (0x8000000000000000 >> bit_index))
    }
    else {
        Err(format!("{} > 63", bit_index))
    }
}



/// In `filter`, once you've chosen the appropriate u64, which particular
/// bit should be flipped?
/// `m` is number of bit sin the filter.
fn bit_index(i: u64, m: NonZeroUsize) -> u8 {
    let reduced = (i as usize) % usize::from(m);
    return (reduced % 64) as u8;
}


/// Which u64 in `filter` should `i` be in?
/// `m` is number of bit sin the filter.
fn u64_index(i: u64, m: NonZeroUsize) -> usize {
    let reduced = (i as usize) % usize::from(m);
    reduced / 64
}


/// Given bit number i, which u64 should it be in and within that u64, which
/// bit should it be
/// `m` is number of bits in the filter.
fn bit_array_indices(i: u64, m: NonZeroUsize) -> (usize, u8) {
    return (u64_index(i, m), bit_index(i, m));
}



#[cfg(test)]
mod tests {
    use super::*;


    /// How many u64s does it take to store `m` bits?
    fn num_u64s(m: NonZeroUsize) -> usize {
        (usize::from(m) - 1)/ 64 + 1
    }


    #[test]
    fn test_num_u64s() {
        for m in 1..=64 {
            assert_eq!(num_u64s(NonZeroUsize::new(m).unwrap()), 1);
        }
        for m in 65..=128 {
            assert_eq!(num_u64s(NonZeroUsize::new(m).unwrap()), 2);
        }
        for m in 129..=192 {
            assert_eq!(num_u64s(NonZeroUsize::new(m).unwrap()), 3);
        }
    }

    #[test]
    fn test_u64_index() {

        /* 0-63 64-127 128-191... */
        let nines = NonZeroUsize::new(9999).unwrap();
        for i in 0..64 {
            assert_eq!(u64_index(i, nines), 0);
        }
        for i in 64..128 {
            assert_eq!(u64_index(i, nines), 1);
        }
        for i in 128..192 {
            assert_eq!(u64_index(i, nines), 2);
        }
        for i in 192..256 {
            assert_eq!(u64_index(i, nines), 3);
        }

        /* 10 bits fit in one u64 */
        for i in 0..9999 {
            assert_eq!(u64_index(i, NonZeroUsize::new(10).unwrap()), 0);
        }

        /* 70 bits fit in two u64's only using 6 bits of the second one */
        let seventy = NonZeroUsize::new(70).unwrap();
        for i in 0..64 {
            assert_eq!(u64_index(i, seventy), 0);
        }
        for i in 64..70 {
            assert_eq!(u64_index(i, seventy), 1);
        }
        for i in 70..(70 + 64) {
            assert_eq!(u64_index(i, seventy), 0);
        }
        for i in (70 + 64)..(70 + 64 + 6) {
            assert_eq!(u64_index(i, seventy), 1);
        }

        /* 129 bits fit in three u64's only using 1 bit of the third one */
        let one29 = NonZeroUsize::new(129).unwrap();
        for i in 0..64 {
            assert_eq!(u64_index(i, one29), 0);
        }
        for i in 64..128 {
            assert_eq!(u64_index(i, one29), 1);
        }
        assert_eq!(u64_index(128, one29), 2);
        for i in 129..(129 + 64) {
            assert_eq!(u64_index(i, one29), 0);
        }
        for i in (129 + 64)..(129 + 2 * 64) {
            assert_eq!(u64_index(i, one29), 1);
        }
        assert_eq!(u64_index(129 + 2 * 64, one29), 2);
        assert_eq!(u64_index(129 + 2 * 64 + 1, one29), 0);
    }


    #[test]
    fn test_bit_index() {
        let nines = NonZeroUsize::new(9999).unwrap();
        for i in 0..64 {
            assert_eq!(bit_index(i, nines), i as u8);
        }
        for i in 64..128 {
            assert_eq!(bit_index(i, nines), (i - 64) as u8);
        }
        for i in 128..192 {
            assert_eq!(bit_index(i, nines), (i - 128) as u8);
        }

        let ten = NonZeroUsize::new(10).unwrap();
        for i in 0..9999 {
            assert_eq!(bit_index(i, ten), (i % 10) as u8);
        }

    }

    #[test]
    fn test_bit_array_indices() {
        let nines = NonZeroUsize::new(9999).unwrap();
        for i in 0..64 {
            assert_eq!(bit_array_indices(i, nines), (0, i as u8));
        }
        for i in 64..128 {
            assert_eq!(bit_array_indices(i, nines), (1, (i - 64) as u8));
        }
        for i in 128..192 {
            assert_eq!(bit_array_indices(i, nines), (2, (i - 128) as u8));
        }
        for i in 0..300 {
            assert_eq!(bit_array_indices(i, NonZeroUsize::new(10).unwrap()), (0, (i % 10) as u8));
        }
    }


    #[test]
    fn test_bit_set() {
        for i in 0..63 {
            assert_eq!(bit_set(0x00000000, i), Ok(false));
        }
        for i in 64..=0xff {
            assert!(bit_set(0x00000000, i).is_err());
        }
        let deadbeef_bits = vec![0, 1, 3, 4, 5, 6, 8, 10, 12, 13, 15, 16, 18,
                19, 20, 21, 22, 24, 25, 26, 28, 29, 30, 31];
        for i in 0..0xff {
            if deadbeef_bits.contains(&i) {
                assert_eq!(bit_set(0xdeadbeef00000000, i), Ok(true));
            }
            else if i < 64 {
                assert_eq!(bit_set(0xdeadbeef00000000, i), Ok(false));
            }
            else {
                assert!(bit_set(0xdeadbeef00000000, i).is_err());
            }
        }
    }

    #[test]
    fn test_filter() {

        /* The bytes of the word "known" */
        let known = "known".bytes().collect::<Vec<u8>>();

        /* The 8 different hashes of the word "known" */
        let known_hashes = vec![1183587150, 2402186983, 4132244288, 3394324783,
                1291789908, 1182111577, 867046547, 3528127662,];
        for hash_num in 0..8 {
            let hash = xxh32::xxh32(&known, hash_num);
            assert_eq!(hash, known_hashes[hash_num as usize]);
        }

        assert_eq!(num_u64s(M_NZ), FILTER_NUM_BITS);
        let mut filter: [u64; FILTER_NUM_BITS] = [0; FILTER_NUM_BITS];

        assert!(!is_in_filter(&known, &filter, M_NZ).unwrap());
        assert!(filter_insert(&known, &mut filter, M_NZ).is_ok());
        assert!(is_in_filter(&known, &filter, M_NZ).unwrap());

        for i in 0..10000 {
            let as_string = i.to_string();
            let as_bytes = as_string.as_bytes();
            assert!(!is_in_filter(&as_bytes, &filter, M_NZ).unwrap());
            assert!(filter_insert(&as_bytes, &mut filter, M_NZ).is_ok());
            assert!(is_in_filter(&as_bytes, &filter, M_NZ).unwrap());
        }
    }


    #[test]
    fn test_set_bit() {
        assert_eq!(set_bit(0x0,  0), Ok(0x8000000000000000));
        assert_eq!(set_bit(0x0,  1), Ok(0x4000000000000000));
        assert_eq!(set_bit(0x0,  2), Ok(0x2000000000000000));
        assert_eq!(set_bit(0x0,  3), Ok(0x1000000000000000));
        assert_eq!(set_bit(0x0,  4), Ok(0x0800000000000000));
        assert_eq!(set_bit(0x0,  5), Ok(0x0400000000000000));
        assert_eq!(set_bit(0x0,  6), Ok(0x0200000000000000));
        assert_eq!(set_bit(0x0,  7), Ok(0x0100000000000000));
        assert_eq!(set_bit(0x0,  8), Ok(0x0080000000000000));
        assert_eq!(set_bit(0x0,  9), Ok(0x0040000000000000));
        assert_eq!(set_bit(0x0, 10), Ok(0x0020000000000000));
        assert_eq!(set_bit(0x0, 11), Ok(0x0010000000000000));
        assert_eq!(set_bit(0x0, 12), Ok(0x0008000000000000));
        assert_eq!(set_bit(0x0, 13), Ok(0x0004000000000000));
        assert_eq!(set_bit(0x0, 14), Ok(0x0002000000000000));
        assert_eq!(set_bit(0x0, 15), Ok(0x0001000000000000));
        assert_eq!(set_bit(0x0, 16), Ok(0x0000800000000000));
        assert_eq!(set_bit(0x0, 17), Ok(0x0000400000000000));
        assert_eq!(set_bit(0x0, 18), Ok(0x0000200000000000));
        assert_eq!(set_bit(0x0, 19), Ok(0x0000100000000000));
        assert_eq!(set_bit(0x0, 20), Ok(0x0000080000000000));
        assert_eq!(set_bit(0x0, 21), Ok(0x0000040000000000));
        assert_eq!(set_bit(0x0, 22), Ok(0x0000020000000000));
        assert_eq!(set_bit(0x0, 23), Ok(0x0000010000000000));
        assert_eq!(set_bit(0x0, 24), Ok(0x0000008000000000));
        assert_eq!(set_bit(0x0, 25), Ok(0x0000004000000000));
        assert_eq!(set_bit(0x0, 26), Ok(0x0000002000000000));
        assert_eq!(set_bit(0x0, 27), Ok(0x0000001000000000));
        assert_eq!(set_bit(0x0, 28), Ok(0x0000000800000000));
        assert_eq!(set_bit(0x0, 29), Ok(0x0000000400000000));
        assert_eq!(set_bit(0x0, 30), Ok(0x0000000200000000));
        assert_eq!(set_bit(0x0, 31), Ok(0x0000000100000000));
        assert_eq!(set_bit(0x0, 32), Ok(0x0000000080000000));
        assert_eq!(set_bit(0x0, 33), Ok(0x0000000040000000));
        assert_eq!(set_bit(0x0, 34), Ok(0x0000000020000000));
        assert_eq!(set_bit(0x0, 35), Ok(0x0000000010000000));
        assert_eq!(set_bit(0x0, 36), Ok(0x0000000008000000));
        assert_eq!(set_bit(0x0, 37), Ok(0x0000000004000000));
        assert_eq!(set_bit(0x0, 38), Ok(0x0000000002000000));
        assert_eq!(set_bit(0x0, 39), Ok(0x0000000001000000));
        assert_eq!(set_bit(0x0, 40), Ok(0x0000000000800000));
        assert_eq!(set_bit(0x0, 41), Ok(0x0000000000400000));
        assert_eq!(set_bit(0x0, 42), Ok(0x0000000000200000));
        assert_eq!(set_bit(0x0, 43), Ok(0x0000000000100000));
        assert_eq!(set_bit(0x0, 44), Ok(0x0000000000080000));
        assert_eq!(set_bit(0x0, 45), Ok(0x0000000000040000));
        assert_eq!(set_bit(0x0, 46), Ok(0x0000000000020000));
        assert_eq!(set_bit(0x0, 47), Ok(0x0000000000010000));
        assert_eq!(set_bit(0x0, 48), Ok(0x0000000000008000));
        assert_eq!(set_bit(0x0, 49), Ok(0x0000000000004000));
        assert_eq!(set_bit(0x0, 50), Ok(0x0000000000002000));
        assert_eq!(set_bit(0x0, 51), Ok(0x0000000000001000));
        assert_eq!(set_bit(0x0, 52), Ok(0x0000000000000800));
        assert_eq!(set_bit(0x0, 53), Ok(0x0000000000000400));
        assert_eq!(set_bit(0x0, 54), Ok(0x0000000000000200));
        assert_eq!(set_bit(0x0, 55), Ok(0x0000000000000100));
        assert_eq!(set_bit(0x0, 56), Ok(0x0000000000000080));
        assert_eq!(set_bit(0x0, 57), Ok(0x0000000000000040));
        assert_eq!(set_bit(0x0, 58), Ok(0x0000000000000020));
        assert_eq!(set_bit(0x0, 59), Ok(0x0000000000000010));
        assert_eq!(set_bit(0x0, 60), Ok(0x0000000000000008));
        assert_eq!(set_bit(0x0, 61), Ok(0x0000000000000004));
        assert_eq!(set_bit(0x0, 62), Ok(0x0000000000000002));
        assert_eq!(set_bit(0x0, 63), Ok(0x0000000000000001));
        for i in 64..=0xff {
            assert!(set_bit(0x0, i).is_err());
        }

        /*
        *   f     f  e b   f   f      f    f
        *   !     !  ! !   !   !      !    !
        * 1101 1110 1010 1101 1011 1110 1110 1111
        * 0123 4567 8901 2345 6789 0123 4567 8901
        *             11 1111 1111 2222 2222 2233
        *   d    e    a    d    b    e    e    f
        */
        assert_eq!(set_bit(0xdeadbeef00000000,  0), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000,  1), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000,  2), Ok(0xfeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000,  3), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000,  4), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000,  5), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000,  6), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000,  7), Ok(0xdfadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000,  8), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000,  9), Ok(0xdeedbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 10), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 11), Ok(0xdebdbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 12), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 13), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 14), Ok(0xdeafbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 15), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 16), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 17), Ok(0xdeadfeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 18), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 19), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 20), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 21), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 22), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 23), Ok(0xdeadbfef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 24), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 25), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 26), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 27), Ok(0xdeadbeff00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 28), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 29), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 30), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 31), Ok(0xdeadbeef00000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 32), Ok(0xdeadbeef80000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 33), Ok(0xdeadbeef40000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 34), Ok(0xdeadbeef20000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 35), Ok(0xdeadbeef10000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 36), Ok(0xdeadbeef08000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 37), Ok(0xdeadbeef04000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 38), Ok(0xdeadbeef02000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 39), Ok(0xdeadbeef01000000));
        assert_eq!(set_bit(0xdeadbeef00000000, 40), Ok(0xdeadbeef00800000));
        assert_eq!(set_bit(0xdeadbeef00000000, 41), Ok(0xdeadbeef00400000));
        assert_eq!(set_bit(0xdeadbeef00000000, 42), Ok(0xdeadbeef00200000));
        assert_eq!(set_bit(0xdeadbeef00000000, 43), Ok(0xdeadbeef00100000));
        assert_eq!(set_bit(0xdeadbeef00000000, 44), Ok(0xdeadbeef00080000));
        assert_eq!(set_bit(0xdeadbeef00000000, 45), Ok(0xdeadbeef00040000));
        assert_eq!(set_bit(0xdeadbeef00000000, 46), Ok(0xdeadbeef00020000));
        assert_eq!(set_bit(0xdeadbeef00000000, 47), Ok(0xdeadbeef00010000));
        assert_eq!(set_bit(0xdeadbeef00000000, 48), Ok(0xdeadbeef00008000));
        assert_eq!(set_bit(0xdeadbeef00000000, 49), Ok(0xdeadbeef00004000));
        assert_eq!(set_bit(0xdeadbeef00000000, 50), Ok(0xdeadbeef00002000));
        assert_eq!(set_bit(0xdeadbeef00000000, 51), Ok(0xdeadbeef00001000));
        assert_eq!(set_bit(0xdeadbeef00000000, 52), Ok(0xdeadbeef00000800));
        assert_eq!(set_bit(0xdeadbeef00000000, 53), Ok(0xdeadbeef00000400));
        assert_eq!(set_bit(0xdeadbeef00000000, 54), Ok(0xdeadbeef00000200));
        assert_eq!(set_bit(0xdeadbeef00000000, 55), Ok(0xdeadbeef00000100));
        assert_eq!(set_bit(0xdeadbeef00000000, 56), Ok(0xdeadbeef00000080));
        assert_eq!(set_bit(0xdeadbeef00000000, 57), Ok(0xdeadbeef00000040));
        assert_eq!(set_bit(0xdeadbeef00000000, 58), Ok(0xdeadbeef00000020));
        assert_eq!(set_bit(0xdeadbeef00000000, 59), Ok(0xdeadbeef00000010));
        assert_eq!(set_bit(0xdeadbeef00000000, 60), Ok(0xdeadbeef00000008));
        assert_eq!(set_bit(0xdeadbeef00000000, 61), Ok(0xdeadbeef00000004));
        assert_eq!(set_bit(0xdeadbeef00000000, 62), Ok(0xdeadbeef00000002));
        assert_eq!(set_bit(0xdeadbeef00000000, 63), Ok(0xdeadbeef00000001));
        for i in 64..=0xff {
            assert!(set_bit(0xdeadbeef00000000, i).is_err());
        }
    }
}


#[derive(Debug)]
#[derive(FromArgs)]
/// Elementary bloom filter
struct Args {
    /// an existing bloom filter's filename or that of one to be created.
    /// `-x` is to emphasize that this can overwrite an existing file.
    #[argh(option, short='x')]
    filter_filename: String,

    /// file to (i)nsert into the filter
    #[argh(option, short='i')]
    file_to_insert: Option<String>,

    /// file to (q)uery for in the filter
    #[argh(option, short='q')]
    file_to_query: Option<String>,

    /// send verbose output to stderr
    #[argh(switch, short='v')]
    verbose: bool,
}


fn fresh_filter() -> [u64; FILTER_NUM_BITS] {
    [0; FILTER_NUM_BITS]
}


fn bytes_or_fail(filename: &str) -> Vec<u8> {
    match fs::read(&filename) {
        Ok(actual_bytes) => {
            actual_bytes
        },
        Err(err) => {
            println!("Unable to read '{}' ({:?})", filename, err);
            process::exit(6);
        }
    }
}


fn regular_file_or_fail(filename: &str) {
    let f_path = Path::new(&filename);
    if !f_path.exists() {
        println!("Cannot access file '{}'", filename);
        process::exit(2);
    }
    if !f_path.is_file() {
        println!("'{}' isn't a regular file", filename);
        process::exit(3);
    }
}


/// Procedure that will exit whole program happily or with error
fn query_existing_filter_and_quit(filter_filename: &str, query_filename: &str) {
    if query_filename.eq(filter_filename) {
        println!("Can't query for a filter in itself");
        process::exit(5);
    }

    regular_file_or_fail(query_filename);

    let filter: Result<Vec<u64>, String> = read_filter_from_disk(&filter_filename).or_else(|err| {
        println!("ERROR: {:?}", err);
        process::exit(16);
    });
    let filter = filter.unwrap();

    match is_in_filter(&bytes_or_fail(query_filename), &filter, M_NZ) {
        Ok(is_in) => {
            if is_in {
                println!("IN");
            }
            else {
                println!("NOT IN");
            }
            process::exit(0);
        },
        Err(err) => {
            println!("ERROR: {}", err);
            process::exit(15);
        },
    }
}


/// Procedure that will exit whole program happily or with error
fn insert_existing_filter_and_quit(
    verbosity: bool,
    filter_filename: &str,
    insert_filename: &str
) {
    if insert_filename.eq(filter_filename) {
        println!("Can't add a filter to itself");
        process::exit(5);
    }

    regular_file_or_fail(insert_filename);

    vprintln!(
        verbosity,
        "Adding file '{}' to existing filter at '{}'",
        insert_filename,
        filter_filename
    );

    match read_filter_from_disk(&filter_filename) {
        Ok(mut filter) => {
            let bytes_to_insert = bytes_or_fail(&insert_filename);
            match filter_insert(&bytes_to_insert, &mut filter, M_NZ) {
                Ok(()) => {
                    match write_filter_to_disk(&filter_filename, &filter) {
                        Ok(()) => {
                            process::exit(0);
                        },
                        Err(err) => {
                            println!("ERROR: {:?}", err);
                            process::exit(16);
                        }
                    }
                },
                Err(err) => {
                    println!("ERROR: {:?}", err);
                    process::exit(7);
                }
            }
        },
        Err(err) => {
            println!("ERROR: {:?}", err);
            process::exit(5);
        }
    }
}


/// Procedure that will exit whole program happily or with error
fn new_filter_and_quit(
    verbosity: bool,
    filter_filename: &str,
    to_add_filename: Option<String>
) {
    let ff_path = Path::new(&filter_filename);
    if ff_path.exists() {
        println!("'{}' already exists", filter_filename);
        process::exit(12);
    }
    vprintln!(verbosity, "Creating a new filter at '{}'", filter_filename);
    let mut filter = fresh_filter();

    let file_to_insert = match to_add_filename {
        Some(ref filename) => {
            regular_file_or_fail(&filename);
            match fs::read(&filename) {
                Ok(actual_bytes) => {
                    Some(actual_bytes)
                },
                Err(err) => {
                    println!("Unable to read '{:?}' ({:?})", to_add_filename.unwrap(), err);
                    process::exit(6);
                }
            }
        },
        None => {
            None
        },
    };

    if file_to_insert.is_some() {
        if let Err(err) = filter_insert(&file_to_insert.unwrap(), &mut filter, M_NZ) {
            println!("ERROR: {:?}", err);
            process::exit(13);
        }
    }

    match write_filter_to_disk(&filter_filename, &filter) {
        Ok(()) => {
            process::exit(0);
        },
        Err(err) => {
            println!(
                "Couldn't write filter to disk at '{}': {:?}",
                &filter_filename,
                err
            );
            process::exit(10);
        }
    };
}


fn main() {
    let args: Args = argh::from_env();

    let ff_path = Path::new(&args.filter_filename);
    let create_new_filter = if ff_path.exists() {
        regular_file_or_fail(&args.filter_filename);
        false
    }
    else {
        true
    };

    if args.file_to_insert.is_some() && args.file_to_query.is_some() {
        println!(
            "Cannot both insert and query for {}/{}",
            args.file_to_insert.unwrap(),
            args.file_to_query.unwrap()
        );
        process::exit(17);
    }

    if create_new_filter {
        if args.file_to_query.is_some() {
            println!(
                "Should not ask if '{}' is in an empty filter you're about to create at {}",
                args.file_to_query.unwrap(),
                args.filter_filename
            );
            process::exit(9);
        }
        let file_to_insert = if args.file_to_insert.is_some() {
            Some(args.file_to_insert.unwrap())
        }
        else {
            None
        };
        new_filter_and_quit(
            args.verbose,
            &args.filter_filename,
            file_to_insert
        );
    }
    else {
        if args.file_to_insert.is_some() {
            insert_existing_filter_and_quit(
                args.verbose,
                &args.filter_filename,
                &args.file_to_insert.unwrap()
            );
        }
        else if args.file_to_query.is_some() {
            query_existing_filter_and_quit(
                &args.filter_filename,
                &args.file_to_query.unwrap()
            );
        }
        else {
            println!("Nothing to do with an existing filter if no file given to query or insert.");
            process::exit(14);
        }
    }
}
