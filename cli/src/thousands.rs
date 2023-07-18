use std::fmt::Display;

pub struct Thousands(pub u64);

impl Display for Thousands {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Thousands(mut value) = *self;
        if value == 0 {
            return write!(formatter, "0");
        }

        let mut buffer = [0u8; 32];
        let mut i_start = 32;
        let mut num_digits = 0;
        while value > 0 {
            i_start -= 1;
            if num_digits > 0 && num_digits % 3 == 0 {
                buffer[i_start] = b',';
                i_start -= 1;
            }
            let (digit, quotient) = (value % 10, value / 10);
            buffer[i_start] = u8::try_from(digit).expect("digits fit in u8") + b'0';
            num_digits += 1;
            value = quotient;
        }
        write!(
            formatter,
            "{}",
            std::str::from_utf8(&buffer[i_start..]).expect("ascii str built manually")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn formatted(value: u64) -> String {
        format!("{}", Thousands(value))
    }

    #[test]
    fn thousands_zero() {
        assert_eq!(formatted(0), "0");
    }

    #[test]
    fn thousands_less_than_one_thousand() {
        assert_eq!(formatted(1), "1");
        assert_eq!(formatted(12), "12");
        assert_eq!(formatted(123), "123");
        assert_eq!(formatted(999), "999");
        assert_eq!(formatted(100), "100");
    }

    #[test]
    fn thousands_one_comma() {
        assert_eq!(formatted(1000), "1,000");
        assert_eq!(formatted(1234), "1,234");
        assert_eq!(formatted(9999), "9,999");
        assert_eq!(formatted(12_345), "12,345");
        assert_eq!(formatted(123_456), "123,456");
        assert_eq!(formatted(123_000), "123,000");
        assert_eq!(formatted(100_000), "100,000");
        assert_eq!(formatted(999_999), "999,999");
    }

    #[test]
    fn thousands_three_comma_club() {
        assert_eq!(formatted(1_000_000_000), "1,000,000,000");
        assert_eq!(formatted(123_456_789_012), "123,456,789,012");
        assert_eq!(formatted(999_999_999_999), "999,999,999,999");
    }

    #[test]
    fn thousands_max_u64() {
        assert_eq!(
            formatted(18446744073709551615),
            "18,446,744,073,709,551,615"
        );
    }
}
