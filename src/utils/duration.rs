use chrono::Duration as ChronoDuration;
use duration_str::parse;

const DEFAULT_EXPIRATION_HOURS: i64 = 24;

pub fn parse_token_expiration(exp_str: &str) -> ChronoDuration {
    parse(exp_str)
        .ok()
        .and_then(|duration| ChronoDuration::from_std(duration).ok())
        .unwrap_or_else(|| {
            eprintln!(
                "Warning: Invalid JWT expiration format '{exp_str}'. Using default {DEFAULT_EXPIRATION_HOURS}h."
            );
            ChronoDuration::hours(DEFAULT_EXPIRATION_HOURS)
        })
}
