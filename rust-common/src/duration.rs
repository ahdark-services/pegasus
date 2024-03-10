use std::num::ParseIntError;
use std::time::Duration;

pub fn parse_go_duration(s: &str) -> Result<Duration, ParseIntError> {
    let len = s.len();

    return if s.ends_with("ms") {
        let ms = s[..len - 2].parse::<u64>()?;
        Ok(Duration::from_millis(ms))
    } else if s.ends_with("s") {
        let sec = s[..len - 1].parse::<u64>()?;
        Ok(Duration::from_secs(sec))
    } else if s.ends_with("m") {
        let min = s[..len - 1].parse::<u64>()?;
        Ok(Duration::from_secs(min * 60))
    } else if s.ends_with("h") {
        let hour = s[..len - 1].parse::<u64>()?;
        Ok(Duration::from_secs(hour * 60 * 60))
    } else {
        Ok(Duration::from_secs(s.parse::<u64>()?))
    };
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_go_duration() {
        assert_eq!(parse_go_duration("1ms").unwrap(), Duration::from_millis(1));
        assert_eq!(parse_go_duration("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_go_duration("1m").unwrap(), Duration::from_secs(60));
        assert_eq!(
            parse_go_duration("1h").unwrap(),
            Duration::from_secs(60 * 60)
        );
        assert_eq!(parse_go_duration("1").unwrap(), Duration::from_secs(1));
    }
}
