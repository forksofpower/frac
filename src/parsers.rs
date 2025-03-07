use std::str::FromStr;

use num::Complex;

/// A generic function to parse a pair from a string separated by a given character.
pub fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    // Find the index of the separator in the string.
    match s.find(separator) {
        None => None, // If no separator is found return None.
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            // If both left and right parts of the string can be parsed into T then return them as a tuple wrapped in a Some.
            (Ok(left), Ok(right)) => Some((left, right)),
            _ => None, // else return None if there is a parsing error on either side.
        },
    }
}

#[allow(dead_code)]
pub fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_parse_pair() {
        assert_eq!(parse_pair::<i32>("", ','), None);
        assert_eq!(parse_pair::<i32>("10,", ','), None);
        assert_eq!(parse_pair::<i32>(",5", ','), None);
        assert_eq!(parse_pair::<i32>("10,5", ','), Some((10, 5)));
        assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
        assert_eq!(parse_pair::<i32>("0.5x", 'x'), None);
        assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
    }

    #[test]
    fn test_parse_complex() {
        assert_eq!(
            parse_complex("1.25,-0.0625"),
            Some(Complex {
                re: 1.25,
                im: -0.0625
            })
        );
        assert_eq!(parse_complex(",-0.0625"), None)
    }
}
