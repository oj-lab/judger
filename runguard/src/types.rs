use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct SoftHardTime(f64, f64);

#[derive(Debug)]
pub struct ParseSoftHardTimeError {
    details: String,
}

impl ParseSoftHardTimeError {
    fn new(msg: &str) -> ParseSoftHardTimeError {
        ParseSoftHardTimeError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ParseSoftHardTimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for ParseSoftHardTimeError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl FromStr for SoftHardTime {
    type Err = ParseSoftHardTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();

        let soft = parts[0]
            .parse::<f64>()
            .map_err(|_| ParseSoftHardTimeError::new("Failed to parse soft time"))?;
        let hard = if parts.len() > 1 {
            parts[1]
                .parse::<f64>()
                .map_err(|_| ParseSoftHardTimeError::new("Failed to parse hard time"))?
        } else {
            soft
        };

        Ok(SoftHardTime(soft, hard))
    }
}
