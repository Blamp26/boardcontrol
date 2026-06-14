use crate::backend::trace::{TraceBackend, TraceEvent};
use crate::board::profile_for;
use crate::error::{Error, Result};
use crate::linux::dev_port::DevPort;
use crate::linux::proc_ioports::superio_ports_available;
use crate::nct::run_sequence;
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
        "detect" => handle_detect(args),
        "nct" => handle_nct(args),
        _ => Err(Error::InvalidArgs(help())),
    }
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

fn help() -> String {
    [
        "usage:",
        "  msi-ml detect --board 7A45",
        "  msi-ml nct detect-chip --backend dev-port --confirm-read",
        "  msi-ml nct init-7a45 --dry-run",
        "  msi-ml nct reset-led --dry-run",
    ]
    .join("\n")
}
