use crate::backend::trace::{TraceBackend, TraceEvent};
use crate::board::profile_for;
use crate::error::{Error, Result};
use crate::linux::dev_port::DevPort;
use crate::linux::dev_port_info::{dev_port_exists, dev_port_metadata_string};
use crate::linux::dmi::{PreflightStatus, evaluate_hardware_read_preflight, read_dmi_info};
use crate::linux::proc_ioports::superio_ports_available;
use crate::nct::allowlist::allowed_change_mask;
use crate::nct::plan::{NctPlanStep, plan_sequence};
use crate::nct::run_sequence;
use crate::nct::sequence::{init_sequence_7a45, reset_led_sequence_7a45};
use crate::nct::superio::read_ldn_reg;
use crate::nct::superio::read_nct6779d_chip_id;

pub fn run() -> Result<()> {
    run_from(std::env::args().skip(1))
}

fn run_from<I>(mut args: I) -> Result<()>
where
    I: Iterator<Item = String>,
{
    let Some(command) = args.next() else {
        return Err(Error::InvalidArgs(help()));
    };

    match command.as_str() {
        "doctor" => handle_doctor(),
        "detect" => handle_detect(args),
        "nct" => handle_nct(args),
        _ => Err(Error::InvalidArgs(help())),
    }
}

fn handle_doctor() -> Result<()> {
    let dmi = read_dmi_info()?;
    let ioports_result = superio_ports_available();
    let dev_port_exists = dev_port_exists();
    let dev_port_meta = dev_port_metadata_string();

    println!("DMI:");
    println!(
        "  board_vendor = {}",
        dmi.board_vendor.as_deref().unwrap_or("unknown")
    );
    println!(
        "  board_name = {}",
        dmi.board_name.as_deref().unwrap_or("unknown")
    );
    println!(
        "  board_version = {}",
        dmi.board_version.as_deref().unwrap_or("unknown")
    );
    println!(
        "  product_name = {}",
        dmi.product_name.as_deref().unwrap_or("unknown")
    );
    println!("  looks_like_msi_7a45 = {}", dmi.looks_like_msi_7a45());

    println!("/proc/ioports:");
    match &ioports_result {
        Ok(available) => {
            println!("  read = yes");
            println!("  004e-004f available = {available}");
        }
        Err(err) => {
            println!("  read = no");
            println!("  004e-004f available = unknown");
            println!("  error = {err}");
        }
    }

    println!("/dev/port:");
    println!("  exists = {dev_port_exists}");
    println!("  metadata = {dev_port_meta}");

    let final_status = match &ioports_result {
        Err(err) => PreflightStatus::Blocked(format!("{err}")),
        Ok(available) => match evaluate_hardware_read_preflight(&dmi, *available) {
            PreflightStatus::Pass if dev_port_exists => PreflightStatus::Pass,
            PreflightStatus::Pass => {
                PreflightStatus::Blocked("dev/port does not exist".to_string())
            }
            PreflightStatus::Blocked(reason) => PreflightStatus::Blocked(reason),
        },
    };

    println!();
    match final_status {
        PreflightStatus::Pass => println!("Hardware read preflight: PASS"),
        PreflightStatus::Blocked(reason) => {
            println!("Hardware read preflight: BLOCKED");
            println!("Reason: {reason}");
        }
    }

    Ok(())
}

fn handle_detect<I>(mut args: I) -> Result<()>
where
    I: Iterator<Item = String>,
{
    let mut board = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--board" => board = args.next(),
            _ => return Err(Error::InvalidArgs(help())),
        }
    }

    let board = board.ok_or_else(|| Error::InvalidArgs(help()))?;
    let supported = profile_for(&board).is_some();
    if supported {
        println!("board {board} supported in trace mode");
        Ok(())
    } else {
        Err(Error::UnsupportedBoard(board))
    }
}

fn handle_nct<I>(mut args: I) -> Result<()>
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        return Err(Error::InvalidArgs(help()));
    };

    if subcommand == "plan-init-7a45" {
        return handle_plan(init_sequence_7a45());
    }
    if subcommand == "plan-reset-led" {
        return handle_plan(reset_led_sequence_7a45());
    }
    if subcommand == "read-reg" {
        return handle_read_reg(args);
    }
    if subcommand == "detect-chip" {
        return handle_detect_chip(args);
    }

    let mut dry_run = false;
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            _ => return Err(Error::InvalidArgs(help())),
        }
    }

    if !dry_run {
        return Err(Error::InvalidArgs(
            "only --dry-run is supported in this MVP".to_string(),
        ));
    }

    let profile = profile_for("7A45").ok_or_else(|| Error::UnsupportedBoard("7A45".to_string()))?;
    let mut backend = TraceBackend::new();

    let sequence = match subcommand.as_str() {
        "init-7a45" => profile.init_sequence(),
        "reset-led" => profile.reset_led_sequence(),
        _ => return Err(Error::InvalidArgs(help())),
    };

    run_sequence(&mut backend, &sequence)?;
    for event in backend.log() {
        match event {
            TraceEvent::Read { ldn, reg, value } => {
                println!("TRACE read  ldn=0x{ldn:02X} reg=0x{reg:02X} value=0x{value:02X}");
            }
            TraceEvent::Write { ldn, reg, value } => {
                println!("TRACE write ldn=0x{ldn:02X} reg=0x{reg:02X} value=0x{value:02X}");
            }
            TraceEvent::Delay { ms } => {
                println!("TRACE delay {ms} ms");
            }
        }
    }

    Ok(())
}

fn handle_plan(sequence: crate::nct::sequence::NctSequence) -> Result<()> {
    let mut backend = TraceBackend::new();
    let plan = plan_sequence(&mut backend, &sequence)?;

    for step in plan {
        match step {
            NctPlanStep::Rmw(rmw) => {
                println!(
                    "PLAN RMW ldn=0x{ldn:02X} reg=0x{reg:02X} current=0x{current:02X} and=0x{and_mask:02X} or=0x{or_mask:02X} new=0x{new_value:02X} changed=0x{changed:02X} allowed=0x{allowed_change_mask:02X} status={status}",
                    ldn = rmw.ldn,
                    reg = rmw.reg,
                    current = rmw.current,
                    and_mask = rmw.and_mask,
                    or_mask = rmw.or_mask,
                    new_value = rmw.new_value,
                    changed = rmw.changed,
                    allowed_change_mask = rmw.allowed_change_mask,
                    status = if rmw.allowed { "allowed" } else { "blocked" },
                );
            }
            NctPlanStep::Delay(ms) => {
                println!("PLAN delay {ms} ms");
            }
        }
    }

    Ok(())
}

fn handle_read_reg<I>(mut args: I) -> Result<()>
where
    I: Iterator<Item = String>,
{
    let mut board = None;
    let mut backend = None;
    let mut ldn = None;
    let mut reg = None;
    let mut confirm_read = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--board" => board = args.next(),
            "--backend" => backend = args.next(),
            "--ldn" => {
                ldn = args
                    .next()
                    .map(|value| parse_u8_value(&value))
                    .transpose()?
            }
            "--reg" => {
                reg = args
                    .next()
                    .map(|value| parse_u8_value(&value))
                    .transpose()?
            }
            "--confirm-read" => confirm_read = true,
            _ => return Err(Error::InvalidArgs(help())),
        }
    }

    if !confirm_read {
        return Err(Error::InvalidArgs(
            "read-reg requires --confirm-read".to_string(),
        ));
    }

    let board = board.ok_or_else(|| Error::InvalidArgs(help()))?;
    if board != "7A45" {
        return Err(Error::UnsupportedBoard(board));
    }

    let backend = backend.ok_or_else(|| Error::InvalidArgs(help()))?;
    if backend != "dev-port" {
        return Err(Error::InvalidArgs(
            "read-reg currently supports only --backend dev-port".to_string(),
        ));
    }

    let ldn = ldn.ok_or_else(|| Error::InvalidArgs(help()))?;
    let reg = reg.ok_or_else(|| Error::InvalidArgs(help()))?;

    if allowed_change_mask(ldn, reg).is_none() {
        return Err(Error::InvalidArgs(format!(
            "LDN=0x{ldn:02X} REG=0x{reg:02X} is not allowlisted"
        )));
    }

    let dmi = read_dmi_info()?;
    if !dmi.looks_like_msi_7a45() {
        return Err(Error::HostDmiMismatch(format!(
            "host DMI does not look like MSI 7A45: vendor={} board={} product={}",
            dmi.board_vendor.as_deref().unwrap_or("unknown"),
            dmi.board_name
                .as_deref()
                .or(dmi.board_version.as_deref())
                .unwrap_or("unknown"),
            dmi.product_name.as_deref().unwrap_or("unknown"),
        )));
    }

    if !superio_ports_available()? {
        return Err(Error::InvalidArgs(
            "/proc/ioports reports 004e-004f as busy".to_string(),
        ));
    }

    let mut port = DevPort::open()?;
    let chip_id = read_nct6779d_chip_id(&mut port)?;
    if !chip_id.is_nct6779d() {
        return Err(Error::UnsupportedBoard(format!(
            "7A45 requires NCT6779D, got id_high=0x{:02X} revision=0x{:02X}",
            chip_id.id_high, chip_id.revision
        )));
    }

    let value = read_ldn_reg(&mut port, ldn, reg)?;
    println!("NCT6779D LDN=0x{ldn:02X} REG=0x{reg:02X} VALUE=0x{value:02X}");
    Ok(())
}

fn handle_detect_chip<I>(mut args: I) -> Result<()>
where
    I: Iterator<Item = String>,
{
    let mut backend = None;
    let mut confirm_read = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--backend" => backend = args.next(),
            "--confirm-read" => confirm_read = true,
            _ => return Err(Error::InvalidArgs(help())),
        }
    }

    if !confirm_read {
        return Err(Error::InvalidArgs(
            "detect-chip requires --confirm-read".to_string(),
        ));
    }

    let backend = backend.ok_or_else(|| Error::InvalidArgs(help()))?;
    if backend != "dev-port" {
        return Err(Error::InvalidArgs(
            "detect-chip currently supports only --backend dev-port".to_string(),
        ));
    }

    let dmi = read_dmi_info()?;
    if !dmi.looks_like_msi_7a45() {
        return Err(Error::HostDmiMismatch(format!(
            "host DMI does not look like MSI 7A45: vendor={} board={} product={}",
            dmi.board_vendor.as_deref().unwrap_or("unknown"),
            dmi.board_name
                .as_deref()
                .or(dmi.board_version.as_deref())
                .unwrap_or("unknown"),
            dmi.product_name.as_deref().unwrap_or("unknown"),
        )));
    }

    if !superio_ports_available()? {
        return Err(Error::InvalidArgs(
            "/proc/ioports reports 004e-004f as busy".to_string(),
        ));
    }

    let mut port = DevPort::open()?;
    let chip_id = read_nct6779d_chip_id(&mut port)?;
    println!(
        "NCT chip id_high=0x{:02X} revision=0x{:02X}",
        chip_id.id_high, chip_id.revision
    );
    if chip_id.is_nct6779d() {
        println!("Detected: NCT6779D");
    } else {
        println!("Detected: unsupported Super I/O");
    }

    Ok(())
}

fn parse_u8_value(input: &str) -> Result<u8> {
    let parsed = if let Some(hex) = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16)
            .map_err(|_| Error::InvalidArgs(format!("invalid numeric value: {input}")))?
    } else {
        input
            .parse::<u16>()
            .map_err(|_| Error::InvalidArgs(format!("invalid numeric value: {input}")))?
    };

    if parsed > u8::MAX as u16 {
        Err(Error::InvalidArgs(format!("value out of range: {input}")))
    } else {
        Ok(parsed as u8)
    }
}

fn help() -> String {
    [
        "usage:",
        "  msi-ml doctor",
        "  msi-ml detect --board 7A45",
        "  msi-ml nct plan-init-7a45",
        "  msi-ml nct plan-reset-led",
        "  msi-ml nct read-reg --board 7A45 --backend dev-port --ldn 0x09 --reg 0xE0 --confirm-read",
        "  msi-ml nct detect-chip --backend dev-port --confirm-read",
        "  msi-ml nct init-7a45 --dry-run",
        "  msi-ml nct reset-led --dry-run",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::parse_u8_value;

    #[test]
    fn parse_u8_accepts_hex_and_decimal() {
        assert_eq!(parse_u8_value("0x09").unwrap(), 9);
        assert_eq!(parse_u8_value("09").unwrap(), 9);
        assert_eq!(parse_u8_value("224").unwrap(), 224);
        assert_eq!(parse_u8_value("0xE0").unwrap(), 224);
    }

    #[test]
    fn parse_u8_rejects_invalid_and_out_of_range_values() {
        assert!(parse_u8_value("0x100").is_err());
        assert!(parse_u8_value("bad").is_err());
    }
}
