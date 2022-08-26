use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
pub struct TwitterSecrets {
    api_key: String,
    api_key_secret: String,
    pub bearer_token: String,
}

#[derive(Deserialize)]
pub struct Secrets {
    pub twitter: TwitterSecrets,
    test_key: String,
}

pub fn extract() -> Result<Secrets, String> {
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
        let secrets = extract().unwrap();
        assert_eq!(secrets.test_key, "test_value");
    }
}
