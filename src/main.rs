fn main() {
    println!("Hello, world!");
}

mod twitter {
    use chrono::prelude::*;

    fn url_tweets(user_id: u32, start_time: DateTime<Utc>) -> String {
        format!(
            "https://api.twitter.com/2/users/{user_id}/tweets?start_time={datetime_iso_8601}",
            user_id = user_id,
            datetime_iso_8601 = start_time.to_rfc3339_opts(SecondsFormat::Millis, true),
        )
    }

    fn url_id_from_username(username: &str) -> String {
        format!(
            "https://api.twitter.com/2/users/by/username?username={username}",
            username = username
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_url_tweets() {
            let user_id = 123;
            let start_time = Utc.ymd(2022, 6, 27).and_hms(1, 30, 0);

            let url = url_tweets(user_id, start_time);
            assert_eq!(
                url,
                "https://api.twitter.com/2/users/123/tweets?start_time=2022-06-27T01:30:00.000Z"
            );
        }

        #[test]
        fn test_url_id_from_username() {
            let user_id = "exampleUser";

            let url = url_id_from_username(user_id);
            assert_eq!(
                url,
                "https://api.twitter.com/2/users/by/username?username=exampleUser"
            );
        }
    }
}

mod secrets {
    use serde::Deserialize;
    use std::process::Command;

    #[derive(Deserialize)]
    struct TwitterSecrets {
        api_key: String,
        api_key_secret: String,
        bearer_token: String,
    }

    #[derive(Deserialize)]
    struct Secrets {
        twitter: TwitterSecrets,
        test_key: String,
    }

    fn extract() -> Result<Secrets, String> {
        let output = Command::new("sops")
            .arg("-d")
            .arg("--output-type")
            .arg("json")
            .arg("src/secrets.yaml")
            .output()
            .expect("failed to execute process");

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }

        let secrets = match serde_json::from_slice(&output.stdout) {
            Ok(secrets) => secrets,
            Err(err) => return Err(format!("Parsing output error: {}", err)),
        };

        Ok(secrets)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_extract() {
            let result = extract();
            match result {
                Ok(secrets) => assert_eq!(secrets.test_key, "test_value"),
                Err(err) => panic!("{}", err),
            }
        }
    }
}
