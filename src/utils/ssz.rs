/*
 * This is a WIP of implementing an alternative 
 * serialization strategy. It attempts to follow Vitalik's
 * "ssz" format here: 
 * https://github.com/ethereum/research/tree/master/py_ssz
 *
 * Currently it does not shorten values to their minimum 
 * length. E.g., a u32 will always be 4 bytes regardless
 * of the value it stores. I believe it should be shortened
 * to the minimum possible length -- so 1_u32 only takes up
 * 1 byte.
 */


extern crate bytes;
extern crate ethereum_types;

// use byteorder::{ BigEndian, WriteBytesExt };
use self::bytes::{ BytesMut, BufMut };
use self::ethereum_types::{ H256, U256 };

pub trait Encodable {
    fn ssz_append(&self, s: &mut SszStream);
}

pub struct SszStream {
    buffer: Vec<u8>
}

impl SszStream {
    pub fn new() -> Self {
        SszStream {
            buffer: Vec::new()
        }
    }

    pub fn append<E>(&mut self, value: &E) -> &mut Self
        where E: Encodable
    {
        value.ssz_append(self);
        self
    }

    fn append_encoded_vec(&mut self, v: &mut Vec<u8>) {
        self.buffer.append(&mut encode_length(v.len()));
        self.buffer.append(v) ;
    }
    
    fn append_encoded_array(&mut self, a: &mut [u8]) {
        let len = a.len();
        self.buffer.append(&mut encode_length(len));
        self.buffer.extend_from_slice(&a[0..len]);
    }

    pub fn drain(self) -> Vec<u8> {
        self.buffer
    }
}

pub fn encode<E>(value: &E) -> Vec<u8>
    where E: Encodable
{
    let mut stream = SszStream::new();
    stream.append(value);
    stream.drain()
}

fn encode_length(len: usize) -> Vec<u8> {
    // Ensure length can fit in 3 bytes (3 * 8 = 24)
    assert!((len as u32) < 2u32.pow(24));
    vec![
        ((len >> 16) & 0xff) as u8,
        ((len >>  8) & 0xff) as u8,
        (        len & 0xff) as u8
    ]
}

/*
 * Implementations for various types
 */

impl Encodable for u32 {
    fn ssz_append(&self, s: &mut SszStream) {
        let mut buf = BytesMut::with_capacity(32/8);
        buf.put_u32_be(*self);
        s.append_encoded_vec(&mut buf.to_vec());
    }
}

impl Encodable for H256 {
    fn ssz_append(&self, s: &mut SszStream) {
        s.append_encoded_vec(&mut self.to_vec());
    }
}

impl Encodable for U256 {
    fn ssz_append(&self, s: &mut SszStream) {
        let mut a = [0; 32];
        self.to_big_endian(&mut a);
        s.append_encoded_array(&mut a);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        pub struct TestStruct {
            pub one: u32,
            pub two: H256,
            pub three: u32,        
        }

        impl Encodable for TestStruct {
            fn ssz_append(&self, s: &mut SszStream) {
                s.append(&self.one);
                s.append(&self.two);
                s.append(&self.three);
            }
        }

        let t = TestStruct {
            one: 1,
            two: H256::zero(),
            three: 100
        };

        let e = encode(&t);
        // TODO: assert some stuff
        println!("{:?}", e);
    }
}
