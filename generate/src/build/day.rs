const SECONDS: i64 = 86_400;
const SHIFT: i64 = 719_468;
const ERA: i64 = 146_097;

pub fn day(at: i64) -> String {
    let days = at.div_euclid(SECONDS) + SHIFT;
    let era = days.div_euclid(ERA);
    let of_era = days - era * ERA;
    let year_of_era = (of_era - of_era / 1_460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted = (5 * of_year + 2) / 153;
    let date = of_year - (153 * shifted + 2) / 5 + 1;
    let month = if shifted < 10 {
        shifted + 3
    } else {
        shifted - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{date:02}")
}
