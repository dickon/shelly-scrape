# Shelly Scrape

WARNING: Pretty much vibe coded using Claude Sonnet.

Scrape data from Shelly power monitoring and push to Influx.

My motivation is to supplment my circuit level home mains consumption with some Shelly
hardware to get better insights. For instance on one circuit I have a router, two switches
which also power 5 cameras, an NVR/NAS and wifi access points, as well as some other
gadgets, so it's hard to tell what's going with influx.

## Getting Started

This Rust application collects power monitoring data from Shelly IoT devices and stores it in InfluxDB for analysis and visualization.

## Installation

Optionally, make sure nmap is installed; it can be used for scanning your 
network for Shelly devices.


```bash

# Clone the repository
git clone https://github.com/dickon/shelly-scrape.git
cd shelly-scrape

# Build the project
cargo build --release
```

## Usage

```bash
# Run the application
cargo run

# Or run the release binary
./target/release/shelly-scrape
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

<!-- Add license information here -->
