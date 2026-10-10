use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutputFormat {
    #[default]
    Mkv,
    Mp4,
}

impl OutputFormat {
    pub const ALL: [Self; 2] = [Self::Mkv, Self::Mp4];
    pub fn extension(self) -> &'static str {
        match self {
            Self::Mkv => "mkv",
            Self::Mp4 => "mp4",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Mkv => "MKV (Matroska)",
            Self::Mp4 => "MP4",
        }
    }
    pub fn muxer(self) -> &'static str {
        match self {
            Self::Mkv => "matroska",
            Self::Mp4 => "mp4",
        }
    }
    /// Conservative playback policy: MP4 copies AAC/MP3; other codecs become AAC.
    pub fn audio_codec(self, source: &str) -> &str {
        match self {
            Self::Mkv => source,
            Self::Mp4 if matches!(source, "aac" | "mp3") => source,
            Self::Mp4 => "aac",
        }
    }
}

/// Coordinates always refer to decoded source pixels, never display pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    /// Normalize display coordinates before calling, so zoom and DPI cannot change the mask.
    pub fn from_normalized_drag(
        start: [f32; 2],
        end: [f32; 2],
        width: u32,
        height: u32,
    ) -> Result<Self> {
        if width < 4 || height < 4 || start.iter().chain(end.iter()).any(|v| !v.is_finite()) {
            return Err(Error::Invalid("Invalid selection coordinates".into()));
        }
        let x = ((start[0].min(end[0]) * width as f32) as u32).clamp(1, width - 3);
        let y = ((start[1].min(end[1]) * height as f32) as u32).clamp(1, height - 3);
        let right = ((start[0].max(end[0]) * width as f32) as u32).clamp(x + 2, width - 1);
        let bottom = ((start[1].max(end[1]) * height as f32) as u32).clamp(y + 2, height - 1);
        Self {
            x,
            y,
            width: right - x,
            height: bottom - y,
        }
        .validate(width, height)
    }

    pub fn validate(self, width: u32, height: u32) -> Result<Self> {
        // delogo needs a one-pixel context boundary on every side.
        if self.x == 0
            || self.y == 0
            || self.width < 2
            || self.height < 2
            || self
                .x
                .checked_add(self.width)
                .is_none_or(|end| end >= width)
            || self
                .y
                .checked_add(self.height)
                .is_none_or(|end| end >= height)
        {
            return Err(Error::Invalid(
                "Select a region at least 2×2 pixels, with a one-pixel margin inside the frame"
                    .into(),
            ));
        }
        Ok(self)
    }

    pub fn filter(self) -> String {
        format!(
            "delogo=x={}:y={}:w={}:h={}:show=0",
            self.x, self.y, self.width, self.height
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Quality {
    #[default]
    High,
    Balanced,
    Compact,
}

impl Quality {
    pub const ALL: [Self; 3] = [Self::High, Self::Balanced, Self::Compact];
    pub fn label(self) -> &'static str {
        match self {
            Self::High => "High quality (CRF 18)",
            Self::Balanced => "Balanced (CRF 23)",
            Self::Compact => "Smaller file (CRF 28)",
        }
    }
    pub fn crf(self) -> &'static str {
        match self {
            Self::High => "18",
            Self::Balanced => "23",
            Self::Compact => "28",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragging_maps_to_source_pixels_in_both_directions() {
        let expected = Region {
            x: 480,
            y: 270,
            width: 960,
            height: 540,
        };
        assert_eq!(
            Region::from_normalized_drag([0.25, 0.25], [0.75, 0.75], 1920, 1080).unwrap(),
            expected
        );
        assert_eq!(
            Region::from_normalized_drag([0.75, 0.75], [0.25, 0.25], 1920, 1080).unwrap(),
            expected
        );
        let clipped = Region::from_normalized_drag([-1.0, -1.0], [2.0, 2.0], 1920, 1080).unwrap();
        assert_eq!(
            clipped,
            Region {
                x: 1,
                y: 1,
                width: 1918,
                height: 1078
            }
        );
        assert!(Region::from_normalized_drag([f32::NAN, 0.0], [1.0, 1.0], 1920, 1080).is_err());
    }

    #[test]
    fn region_rejects_overflow_and_missing_context() {
        for region in [
            Region {
                x: u32::MAX,
                y: 1,
                width: 4,
                height: 4,
            },
            Region {
                x: 0,
                y: 1,
                width: 4,
                height: 4,
            },
            Region {
                x: 98,
                y: 1,
                width: 2,
                height: 4,
            },
            Region {
                x: 1,
                y: 1,
                width: 1,
                height: 4,
            },
        ] {
            assert!(region.validate(100, 100).is_err());
        }
        assert!(
            Region {
                x: 1,
                y: 1,
                width: 98,
                height: 98
            }
            .validate(100, 100)
            .is_ok()
        );
    }
}
