use anyhow::Result;
use clap::Parser;
use std::process::Command;
use tracing::{info, warn, debug};

#[derive(Debug, Clone)]
struct ShellyDevice {
    ip: String,
    hostname: Option<String>,
}

#[derive(Parser)]
#[command(name = "shelly-scrape")]
#[command(about = "Scrape data from Shelly power monitoring and push to Influx")]
struct Args {
    /// Shelly device IP address (use --discover to find devices automatically)
    #[arg(short, long)]
    shelly_ip: Option<String>,
    
    /// Automatically discover Shelly devices using nmap
    #[arg(short, long)]
    discover: bool,
    
    /// Network range to scan for Shelly devices (e.g., 192.168.1.0/24)
    #[arg(short, long, default_value = "192.168.1.0/24")]
    network: String,
    
    /// InfluxDB URL
    #[arg(short, long, default_value = "http://localhost:8086")]
    influx_url: String,
    
    /// InfluxDB database name
    #[arg(long, default_value = "shelly_data")]
    database: String,
    
    /// Scrape interval in seconds
    #[arg(long, default_value = "60")]
    interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("Starting Shelly scraper");
    
    // Discover or use specified Shelly devices
    let shelly_devices = if args.discover {
        info!("Discovering Shelly devices on network: {}", args.network);
        discover_shelly_devices(&args.network).await?
    } else if let Some(ip) = &args.shelly_ip {
        vec![ShellyDevice { ip: ip.clone(), hostname: None }]
    } else {
        anyhow::bail!("Either specify --shelly-ip or use --discover to find devices");
    };
    
    if shelly_devices.is_empty() {
        warn!("No Shelly devices found!");
        return Ok(());
    }
    
    info!("Found {} Shelly device(s):", shelly_devices.len());
    for device in &shelly_devices {
        if let Some(hostname) = &device.hostname {
            info!("  {} ({})", device.ip, hostname);
        } else {
            info!("  {}", device.ip);
        }
    }
    info!("InfluxDB URL: {}", args.influx_url);
    info!("Database: {}", args.database);
    info!("Interval: {}s", args.interval);
    
    // Main scraping loop
    loop {
        for device in &shelly_devices {
            match scrape_and_push(&args, &device.ip).await {
                Ok(_) => {
                    if let Some(hostname) = &device.hostname {
                        info!("Successfully scraped and pushed data from {} ({})", device.ip, hostname);
                    } else {
                        info!("Successfully scraped and pushed data from {}", device.ip);
                    }
                },
                Err(e) => {
                    if let Some(hostname) = &device.hostname {
                        warn!("Error during scrape from {} ({}): {}", device.ip, hostname, e);
                    } else {
                        warn!("Error during scrape from {}: {}", device.ip, e);
                    }
                },
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(args.interval)).await;
    }
}

async fn discover_shelly_devices(network: &str) -> Result<Vec<ShellyDevice>> {
    info!("Running nmap scan on network: {}", network);
    
    // Run nmap to discover devices on ports commonly used by Shelly devices
    let output = Command::new("nmap")
        .args([
            "-sn", // Ping scan only
            network,
        ])
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("nmap command failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    let nmap_output = String::from_utf8_lossy(&output.stdout);
    debug!("nmap output: {}", nmap_output);
    
    // Extract IP addresses and hostnames from nmap output
    let mut discovered_devices = Vec::new();
    for line in nmap_output.lines() {
        if line.contains("Nmap scan report for") {
            if let Some((ip, hostname)) = extract_device_info_from_nmap_line(line) {
                // Test if this might be a Shelly device by checking port 80
                if is_potential_shelly_device(&ip).await {
                    discovered_devices.push(ShellyDevice { ip, hostname });
                }
            }
        }
    }
    
    Ok(discovered_devices)
}

fn extract_device_info_from_nmap_line(line: &str) -> Option<(String, Option<String>)> {
    // Parse "Nmap scan report for 192.168.1.100" or "Nmap scan report for hostname (192.168.1.100)"
    if let Some(ip_start) = line.rfind('(') {
        if let Some(ip_end) = line.rfind(')') {
            let ip = line[ip_start + 1..ip_end].to_string();
            // Extract hostname from "Nmap scan report for hostname (192.168.1.100)"
            let hostname_part = &line[0..ip_start].trim();
            if let Some(hostname_start) = hostname_part.rfind(' ') {
                let hostname = hostname_part[hostname_start + 1..].trim().to_string();
                if !hostname.is_empty() && hostname != ip {
                    return Some((ip, Some(hostname)));
                }
            }
            return Some((ip, None));
        }
    } else if let Some(ip_part) = line.split_whitespace().last() {
        // Simple case: "Nmap scan report for 192.168.1.100"
        if ip_part.chars().next().unwrap_or('a').is_ascii_digit() {
            return Some((ip_part.to_string(), None));
        }
    }
    None
}

async fn is_potential_shelly_device(ip: &str) -> bool {
    // Try to connect to the Shelly device's web interface
    let client = reqwest::Client::new();
    let url = format!("http://{}/shelly", ip);
    
    match client.get(&url).timeout(std::time::Duration::from_secs(3)).send().await {
        Ok(response) => {
            // Check if response looks like a Shelly device
            if let Ok(text) = response.text().await {
                text.to_lowercase().contains("shelly")
            } else {
                false
            }
        }
        Err(_) => {
            // Also try the status endpoint
            let status_url = format!("http://{}/status", ip);
            match client.get(&status_url).timeout(std::time::Duration::from_secs(3)).send().await {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            }
        }
    }
}

async fn scrape_and_push(args: &Args, shelly_ip: &str) -> Result<()> {
    // TODO: Implement Shelly API scraping
    // TODO: Implement InfluxDB pushing
    
    println!("Scraping data from Shelly device at {}", shelly_ip);
    println!("Would push to InfluxDB at {}", args.influx_url);
    
    Ok(())
}
