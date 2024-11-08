use std::io::Write;

use std::sync::Once;

static START: Once = Once::new();

fn prompt(text: &str) -> String {
    START.call_once(|| {
        let cred_page = "https://zividlabs.atlassian.net/wiki/spaces/CI/pages/4070736030/";
        println!("Use credentials from {}", cred_page);
    });
    print!("Enter {}: ", text);
    std::io::stdout().flush().expect("Oups");

    let mut response = String::new();
    std::io::stdin()
        .read_line(&mut response)
        .expect("Failed to get input");

    let value = response.trim_end().to_string();
    std::env::set_var(text, &value);
    return value;
}


fn read_from_env_or_prompt(env_variable: &str) -> String
{
    let env_value = std::env::var(env_variable);
    match env_value {
        Ok(value) => value,
        Err(_) => prompt(env_variable)
    }
}

fn read_s3_hostname() -> String {
    read_from_env_or_prompt("S3_HOSTNAME")
}

fn read_s3_access_key() -> String {
    read_from_env_or_prompt("S3_ACCESSKEY")
}

fn read_s3_secret_key() -> String {
    read_from_env_or_prompt("S3_SECRETKEY")
}

fn read_s3_bucket_name() -> String {
    read_from_env_or_prompt("S3_BUCKET")
}

pub struct S3Config {
    pub hostname: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
}

pub fn read_credentials() -> S3Config {
    S3Config {
        hostname: read_s3_hostname(),
        access_key: read_s3_access_key(),
        secret_key: read_s3_secret_key(),
        bucket_name: read_s3_bucket_name(),
    }
}

