//! Types convertable to integers

use get_slice::GetSlice;

/// Parse the string to a integer using a given radix
pub trait ToNum {
    fn to_num_radix(&self, radix: u32) -> u32;
    fn to_num_radix_signed(&self, radix: u32) -> i32;
    fn to_num(&self) -> u32;
    fn to_num_signed(&self) -> i32;
}

impl ToNum for str {
    fn to_num_radix(&self, radix: u32) -> u32 {
        if radix == 0 {
            return 0;
        }

        let mut num = 0;
        for c in self.chars() {
            let digit;
            if c >= '0' && c <= '9' {
                digit = c as u32 - '0' as u32
            } else if c >= 'A' && c <= 'Z' {
                digit = c as u32 - 'A' as u32 + 10
            } else if c >= 'a' && c <= 'z' {
                digit = c as u32 - 'a' as u32 + 10
            } else {
                break;
            }

            if digit >= radix {
                break;
            }

            num *= radix;
            num += digit;
        }

        num
    }

    /// Parse the string as a signed integer using a given radix
    fn to_num_radix_signed(&self, radix: u32) -> i32 {
        if self.starts_with('-') {
            -(self.get_slice(Some(1), None).to_num_radix(radix) as i32)
        } else {
            self.to_num_radix(radix) as i32
        }
    }

    /// Parse it as a unsigned integer in base 10
    fn to_num(&self) -> u32 {
        self.to_num_radix(10)
    }

    /// Parse it as a signed integer in base 10
    fn to_num_signed(&self) -> i32 {
        self.to_num_radix_signed(10)
    }
}
