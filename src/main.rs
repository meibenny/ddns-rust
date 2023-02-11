use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Cli {
    /// The path to the config file
    config_file: std::path::PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigDomainsStruct {
    domain: String,
    dns_entry_type: String,
    subdomain: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    secretkey: String,
    apikey: String,
    domains: Vec<ConfigDomainsStruct>
}

#[derive(Serialize, Deserialize, Debug)]
struct PorkbunUpdateDNSStruct {
    secretapikey: String,
    apikey: String,
    content: String,
    ttl: String
}

#[derive(Serialize, Deserialize, Debug)]
struct PorkbunQueryDNSRequest {
    secretapikey: String,
    apikey: String
}

#[derive(Serialize, Deserialize, Debug)]
struct PorkbunQueryDNSRecord {
    id: String,
    name: String,
    #[serde(rename="type")]
    type_: String,
    content: String,
    ttl: String,
    prio: String,
    notes: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
struct PorkbunQueryDNSResponse {
    status: String,
    records: Vec<PorkbunQueryDNSRecord>
}

#[derive(Serialize, Deserialize)]
struct PorkbunUpdateDNSResponse {
    status: String,
    message: Option<String>
}

fn get_current_ip() -> Result<String, reqwest::Error> {
    let url = "https://checkip.amazonaws.com/";
    let resp = reqwest::blocking::get(url)?;
    let ip = resp.text()?;
    // let ip = "192.168.0.140".to_string();
    Ok(ip)
}

fn get_current_dns_entry(config: &Config) -> Result<String, reqwest::Error> {
    let ref secretkey = config.secretkey;
    let ref apikey = config.apikey;
    let ref domain_config = config.domains[0];
    let porkbun_payload = PorkbunQueryDNSRequest {
        secretapikey: String::from(secretkey),
        apikey: String::from(apikey)
    };
    let url = &format!(
        "https://porkbun.com/api/json/v3/dns/retrieveByNameType/{}/{}/{}",
        domain_config.domain, domain_config.dns_entry_type, domain_config.subdomain
    );
    let client = reqwest::blocking::Client::new();
    let resp = client.post(url)
        .json(&porkbun_payload)
        .send()?;
    //println!("{}", &resp.text()?);
    let parsed_resp = resp.json::<PorkbunQueryDNSResponse>()?;
    Ok(parsed_resp.records[0].content.to_string())
}

fn update_dns_entry(
    current_ip: &String,
    current_dns_entry: &String,
    config: &Config
) -> Result<String, reqwest::Error> {
    if current_ip.eq(current_dns_entry) {
       return Ok(
           format!(
               "Current IP: {}, identical to current DNS Entry, {}. Not updating.",
               current_ip, current_dns_entry
           ).to_string()
       )
    }
    let ref secretkey = config.secretkey;
    let ref apikey = config.apikey;
    let ref domain_config = config.domains[0];
    let url = &format!(
        "https://porkbun.com/api/json/v3/dns/editByNameType/{}/{}/{}",
        domain_config.domain, domain_config.dns_entry_type, domain_config.subdomain
    );
    let porkbun_payload = PorkbunUpdateDNSStruct {
        secretapikey: String::from(secretkey),
        apikey: String::from(apikey),
        content: String::from(current_ip),
        ttl: String::from("600")
    };
    let client = reqwest::blocking::Client::new();
    let resp = client.post(url)
        .json(&porkbun_payload)
        .send()?;
    let parsed_resp = resp.json::<PorkbunUpdateDNSResponse>()?;
    if (parsed_resp.status).eq("SUCCESS") {
        return Ok(format!("Upddated DNS Entry to {}", current_ip).to_string())
    } else {
        return Ok(format!("Could not update DNS Entry. {} -> {}", current_dns_entry, current_ip).to_string())
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let config_path = &args.config_file;
    let config_contents = std::fs::read_to_string(config_path)
        .with_context(|| format!("could not read config '{}'", config_path.display()))?;
    let parsed_config: Config = serde_json::from_str(&config_contents).unwrap();
    let current_ip_result = get_current_ip();
    let current_ip = match current_ip_result {
       Ok(ip) => ip,
       Err(error) => panic!("Could not retrieve current IP: {:?}", error),
    };
    let current_entry = get_current_dns_entry(&parsed_config).unwrap();
    let update_result = update_dns_entry(&current_ip, &current_entry, &parsed_config).unwrap();
    println!("{}", update_result);
    Ok(())
}
