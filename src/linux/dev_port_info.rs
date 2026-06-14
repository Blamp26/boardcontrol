use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn dev_port_exists() -> bool {
    Path::new("/dev/port").exists()
}

pub fn dev_port_metadata_string() -> String {
    match fs::metadata("/dev/port") {
        Ok(metadata) => {
            let readonly = metadata.permissions().readonly();
            #[cfg(unix)]
            let mode = metadata.permissions().mode() & 0o777;
            #[cfg(unix)]
            {
                format!("exists=true readonly={readonly} mode={mode:04o}")
            }
            #[cfg(not(unix))]
            {
                format!("exists=true readonly={readonly}")
            }
        }
        Err(err) => format!("exists=false error={err}"),
    }
}
