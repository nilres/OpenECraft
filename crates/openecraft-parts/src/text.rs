//! Text helpers.

/// Comparison that orders "-K2" before "-K10".
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    fn chunks(s: &str) -> Vec<(bool, String)> {
        let mut out: Vec<(bool, String)> = Vec::new();
        for c in s.chars() {
            let d = c.is_ascii_digit();
            match out.last_mut() {
                Some((ld, buf)) if *ld == d => buf.push(c),
                _ => out.push((d, c.to_string())),
            }
        }
        out
    }
    let (ca, cb) = (chunks(a), chunks(b));
    for (x, y) in ca.iter().zip(cb.iter()) {
        let ord = if x.0 && y.0 {
            let nx: u64 = x.1.parse().unwrap_or(0);
            let ny: u64 = y.1.parse().unwrap_or(0);
            nx.cmp(&ny).then_with(|| x.1.cmp(&y.1))
        } else {
            x.1.cmp(&y.1)
        };
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }
    ca.len().cmp(&cb.len())
}

/// `text` matches `pattern` with `*` (any text) and `?` (one character), case-sensitive.
/// Runs in time `pattern × text` without recursion, so long texts and many `*` stay
/// cheap (the servers match names of requests against the patterns of grants).
pub fn wildcard(pattern: &str, text: &str) -> bool {
    let (p, t): (Vec<char>, Vec<char>) = (pattern.chars().collect(), text.chars().collect());
    let (mut pi, mut ti) = (0, 0);
    // The last `*` and where in the text its match ends for now.
    let mut star: Option<(usize, usize)> = None;
    while ti < t.len() {
        match p.get(pi) {
            Some('*') => {
                star = Some((pi, ti));
                pi += 1;
            }
            Some(c) if *c == '?' || *c == t[ti] => {
                pi += 1;
                ti += 1;
            }
            _ => match star {
                // The `*` takes one more character.
                Some((sp, st)) => {
                    star = Some((sp, st + 1));
                    pi = sp + 1;
                    ti = st + 1;
                }
                None => return false,
            },
        }
    }
    p[pi..].iter().all(|c| *c == '*')
}

/// Date `YYYY-MM-DD` (UTC) of a Unix time.
pub fn date_from_unix(secs: u64) -> String {
    // Civil date from days since 1970-01-01 (H. Hinnant's algorithm).
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Unix time of an ISO 8601 date or date and time (`2026-10-23`,
/// `2026-10-23T08:30:00+02:00`); the time zone is ignored. `None` if it is no date.
pub fn unix_from_date(text: &str) -> Option<u64> {
    let text = text.trim();
    let date = text.get(..10)?;
    let num = |s: &str| s.parse::<i64>().ok();
    let (y, m, d) = (num(date.get(..4)?)?, num(date.get(5..7)?)?, num(date.get(8..10)?)?);
    if date.as_bytes()[4] != b'-' || date.as_bytes()[7] != b'-' || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let time = text.get(11..19).and_then(|t| Some(num(t.get(..2)?)? * 3600 + num(t.get(3..5)?)? * 60 + num(t.get(6..8)?)?)).unwrap_or(0);
    // Days since 1970-01-01 of a civil date (H. Hinnant's algorithm).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    u64::try_from(days * 86_400 + time).ok()
}

#[cfg(test)]
mod tests;
