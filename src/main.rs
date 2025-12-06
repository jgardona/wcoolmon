use anyhow::Context;
use clap::Parser;
use hidapi::HidApi;
use std::thread;
use std::time::Duration;
use sysinfo::Components;

const K10TEMP: &str = "k10temp";
const TCTL: &str = "tctl";
const PACKAGE: &str = "package";

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// HID device vendor Id
    #[arg(short, long, default_value_t = 0xaa88)]
    vendor_id: u16,
    /// HID device product Id
    #[arg(short, long, default_value_t = 0x8666)]
    product_id: u16,
    /// Time interval to update temperature info
    #[arg(short, long, default_value_t = 1000, value_parser = clap::value_parser!(u64).range(500..=2000))]
    interval: u64,
    /// Verbose. Prints temperature info in C⁰ to stdout
    #[arg(short = 'r', long, default_value_t = false)]
    verbose: bool,
}

fn get_cpu_temp(components: &Components) -> Option<f32> {
    components
        .iter()
        .find(|c| {
            let label = c.label().to_lowercase();
            label.contains(K10TEMP) || label.contains(TCTL) || label.contains(PACKAGE)
        })
        .map(|c| c.temperature())?
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let api = HidApi::new().context("API HID failed to initialize")?;
    let device = api.open(args.vendor_id, args.product_id).context(format!(
        "Device {:04x}:{:04x} not found",
        args.vendor_id, args.product_id
    ))?;
    println!("Connected to Device: {:04x}", args.product_id);
    let mut components = Components::new_with_refreshed_list();
    loop {
        components.refresh(true);
        let temp = get_cpu_temp(&components).unwrap_or(0.0);
        let temp_byte = temp as u8;
        let command = [0u8, temp_byte];
        match device.write(&command) {
            Ok(_) => {
                if args.verbose {
                    println!("Temperature: {temp_byte}⁰C");
                }
            }
            Err(e) => {
                eprintln!("I/O HID failure: {e}");
                break;
            }
        }
        thread::sleep(Duration::from_millis(args.interval));
    }
    Ok(())
}
