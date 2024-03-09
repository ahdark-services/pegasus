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
