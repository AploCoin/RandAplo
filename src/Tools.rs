
use num_bigint::BigUint;

pub fn biguint_into_u64(number:BigUint) -> u64{
    let bytes_vector = number.to_bytes_le();
    let mut bytes:[u8;8] = [0;8];    
    if bytes_vector.len() > 0{
        let mut n = 0;
        while n<bytes_vector.len()&&
                n<8{
            bytes[n] = bytes_vector[n];
            n += 1;
        }

    }
    return u64::from_le_bytes(bytes);
}

pub fn biguint_into_u32(number:BigUint) -> u32{
    let bytes_vector = number.to_bytes_le();
    let mut bytes:[u8;4] = [0;4];    
    if bytes_vector.len() > 0{
        let mut n = 0;
        while n<bytes_vector.len()&&
                n<4{
            bytes[n] = bytes_vector[n];
            n += 1;
        }

    }
    return u32::from_le_bytes(bytes);
}

pub fn biguint_into_u16(number:BigUint) -> u16{
    let bytes_vector = number.to_bytes_le();
    let mut bytes:[u8;2] = [0;2];    
    if bytes_vector.len() > 0{
        let mut n = 0;
        while n<bytes_vector.len()&&
                n<2{
            bytes[n] = bytes_vector[n];
            n += 1;
        }

    }
    return u16::from_le_bytes(bytes);
}

pub fn biguint_into_u8(number:BigUint) -> u8{
    let bytes_vector = number.to_bytes_be();  
    return bytes_vector[bytes_vector.len()-1];
}