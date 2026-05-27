use crate::DimError;

#[derive(Debug)]
pub(crate) enum ChangeDirection {
    Increase,
    Decrease,
    Absolute,
}

#[derive(Debug)]
pub(crate) enum Magnitude {
    Percentage(f32),
    Absolute(u32),
}

#[derive(Debug)]
pub(crate) struct Change {
    pub(crate) direction: ChangeDirection,
    pub(crate) magnitude: Magnitude,
}

impl std::str::FromStr for Change {
    type Err = DimError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (direction, magnitude) = if let Some(value) = input.strip_prefix('+') {
            (ChangeDirection::Increase, value)
        } else if let Some(value) = input.strip_prefix('-') {
            (ChangeDirection::Decrease, value)
        } else {
            (ChangeDirection::Absolute, input)
        };

        if let Some(percentage) = magnitude.strip_suffix("%") {
            let percentage = percentage.parse::<f32>()?;
            if percentage > 100.0 {
                return Err(DimError::InvalidPercentage);
            }
            Ok(Change {
                direction,
                magnitude: Magnitude::Percentage(percentage),
            })
        } else {
            let magnitude = magnitude.parse::<u32>()?;
            Ok(Change {
                direction,
                magnitude: Magnitude::Absolute(magnitude),
            })
        }
    }
}
