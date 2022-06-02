use chrono::prelude::*;

fn main() {
    println!("Hello, world!");
}

fn create_url(user_id: u32, start_time: DateTime<Utc>) -> String {
    format!(
        "https://api.twitter.com/2/users/{user_id}/tweets?start_time={datetime_iso_8601}",
        user_id = user_id,
        datetime_iso_8601 = start_time.to_rfc3339_opts(SecondsFormat::Millis, true),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_url() {
        let user_id = 123;
        let start_time = Utc.ymd(2022, 6, 27).and_hms(1, 30, 0);

        let url = create_url(user_id, start_time);
        assert_eq!(
            url,
            "https://api.twitter.com/2/users/123/tweets?start_time=2022-06-27T01:30:00.000Z"
        );
    }
}
