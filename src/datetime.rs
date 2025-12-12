pub struct DateTime {
        pub year:   i32,
        pub month:  u8,
        pub day:    u8,
        pub hour:   u8,
        pub minute: u8,
        pub second: u8,
}

impl DateTime {
        pub fn new(timestamp: u64, offset: i32) -> Self {
                let ts = (timestamp as i64) + (offset as i64);

                // Split into days and seconds within the day
                let days = ts.div_euclid(86_400);
                let seconds = ts.rem_euclid(86_400);

                // Shift epoch from 1970-01-01 to 0000-03-01
                let days = days + 719468;

                let era = (if days >= 0 { days } else { days - 146096 }) / 146097;
                let doe = days - era * 146097;
                let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
                let y = yoe + era * 400;
                let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
                let mp = (5 * doy + 2) / 153;
                let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
                let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8;
                let year = y + (if month <= 2 { 1 } else { 0 });

                let hour = (seconds / 3600) as u8;
                let minute = ((seconds % 3600) / 60) as u8;
                let second = (seconds % 60) as u8;

                Self {
                        year: year as i32,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                }
        }

        pub fn format(&self) -> String {
                format!(
                        "{:04}{:02}{:02}-{:02}{:02}{:02}",
                        self.year, self.month, self.day, self.hour, self.minute, self.second
                )
        }
}
