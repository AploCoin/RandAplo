use md5;
use num_bigint::BigUint;

pub fn murmur2(data: &mut [u8], data_length: u64) -> u32{
    let m: u32 = 0x5bd1e995;
    let seed: u32 = 0;
    let r = 24;

    let mut h: u32 = seed ^ (data_length as u32);
    let mut k: u32 = 0;

    let mut round:usize = 0;
    while data_length >= ((round as u64)*4)+4{
        k = data[round*4] as u32;
        k |= (data[(round*4)+1] as u32)<<8;
        k |= (data[(round*4)+2] as u32)<<16;
        k |= (data[(round*4)+3] as u32)<<24;

        k *= m;
        k ^= k >> r;
        k *= m;

        h *= m;
        h ^= k;

        round += 1;
    }

    let length_diff = data_length - ((round as u64)*4);

    match length_diff{
        1 => {
            h ^= data[(data_length-1) as usize] as u32;
            h *= m;
        }
        2 => {h ^= (data[(data_length-1) as usize] as u32)<<8;}
        3 => {h ^= (data[(data_length-1) as usize] as u32)<<16;}
        _ => {}
    }
    h ^= h>>13;
    h *= m;
    h ^= h>>15;

    return h
}

fn transform_u32_to_array_of_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}

pub struct MurMur2RNG{
    seed:u32,
}

impl MurMur2RNG{
    pub fn generate(&mut self) -> u32{
        let mut converted = transform_u32_to_array_of_u8(self.seed);
        self.seed = murmur2(&mut converted,4);
        return self.seed;
    }

    pub fn get_generator(data: &mut [u8], data_length: u64)->MurMur2RNG{
        let seed = murmur2(data,data_length);
        return MurMur2RNG{seed:seed,};
    }
}


pub struct MD5RNG{
    seed:[u8;16],
}

impl MD5RNG {

    pub fn get_generator(data:&mut [u8])->MD5RNG{
        let hash = md5::compute(data);
        let seed:[u8;16] = hash.into();
        return MD5RNG{seed:seed,};
    }
    pub fn generate(&mut self) -> BigUint {
        let hash = md5::compute(self.seed);
        self.seed = hash.into();
        //println!("{:?}",self.seed);
        return BigUint::from_bytes_be(&self.seed);
    }

}