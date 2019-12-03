use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiagnosticError {
    #[cfg(unix)]
    #[error("while performing a UNIX syscall {0}")]
    UnixError(#[source] nix::Error),
    #[error("unknown diagnostic error")]
    UnknownError,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub open_files_limit: Option<u64>,
    pub cpu_usage_limit: Option<u64>,
}

impl Diagnostic {
    pub fn new() -> Result<Self, DiagnosticError> {
        #[cfg(unix)]
        {
            Ok(Self {
                open_files_limit: Some(getrlimit(libc::RLIMIT_NOFILE)?),
                cpu_usage_limit: Some(getrlimit(libc::RLIMIT_CPU)?),
            })
        }
        #[cfg(not(unix))]
        {
            Ok(Self {
                open_files_limit: None,
                cpu_usage_limit: None,
            })
        }
    }
}

impl Display for Diagnostic {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            formatter,
            "limit for open files (RLIMIT_NOFILE): {}",
            self.open_files_limit
                .map(|v| v.to_string())
                .unwrap_or("N/A".to_string())
        )?;
        write!(
            formatter,
            "; limit for CPU usage (RLIMIT_CPU): {}",
            self.cpu_usage_limit
                .map(|v| v.to_string())
                .unwrap_or("N/A".to_string())
        )
    }
}

#[cfg(target_os = "macos")]
type RlimitResource = i32;

#[cfg(all(unix, not(target_os = "macos")))]
type RlimitResource = u32;

#[cfg(unix)]
fn getrlimit(resource: RlimitResource) -> Result<u64, DiagnosticError> {
    use libc::rlimit;

    let mut limits = rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    let retcode = unsafe { libc::getrlimit(resource, &mut limits as *mut rlimit) };
    nix::errno::Errno::result(retcode).map_err(DiagnosticError::UnixError)?;

    Ok(limits.rlim_cur)
}