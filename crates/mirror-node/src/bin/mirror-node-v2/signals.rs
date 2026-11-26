//! Scoped Linux graceful-shutdown handlers for the finite provider only.
//! The handler performs one lock-free atomic store and no I/O/allocation.

use super::errors::Failure;

#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "linux")]
static STOP: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "linux")]
extern "C" fn request_shutdown(_: libc::c_int) {
    // AtomicBool is lock-free on supported targets. Do not add formatting,
    // allocation, locks, cleanup or any other handler-side work here.
    STOP.store(true, Ordering::Relaxed);
}

#[cfg(target_os = "linux")]
pub(super) struct Signals {
    previous_int: libc::sigaction,
    previous_term: libc::sigaction,
}

#[cfg(target_os = "linux")]
impl Signals {
    pub(super) fn install() -> Result<Self, Failure> {
        // This private guard is installed once by the single-threaded serve
        // command. Other operator commands never alter process dispositions.
        STOP.store(false, Ordering::Relaxed);
        // SAFETY: zero initializes libc's plain C sigaction storage, whose
        // function/flags/mask fields are filled before use. All pointers passed
        // below refer to live, correctly aligned values of the libc ABI type.
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = request_shutdown as *const () as libc::sighandler_t;
            action.sa_flags = 0;
            if libc::sigemptyset(&mut action.sa_mask) != 0
                || libc::sigaddset(&mut action.sa_mask, libc::SIGINT) != 0
                || libc::sigaddset(&mut action.sa_mask, libc::SIGTERM) != 0
            {
                return Err(Failure::new("signal_setup"));
            }
            let mut previous_int: libc::sigaction = std::mem::zeroed();
            let mut previous_term: libc::sigaction = std::mem::zeroed();
            if libc::sigaction(libc::SIGINT, &action, &mut previous_int) != 0 {
                return Err(Failure::new("signal_setup"));
            }
            if libc::sigaction(libc::SIGTERM, &action, &mut previous_term) != 0 {
                let _ = libc::sigaction(libc::SIGINT, &previous_int, std::ptr::null_mut());
                return Err(Failure::new("signal_setup"));
            }
            Ok(Self {
                previous_int,
                previous_term,
            })
        }
    }

    pub(super) fn requested(&self) -> bool {
        STOP.load(Ordering::Relaxed)
    }
}

#[cfg(target_os = "linux")]
impl Drop for Signals {
    fn drop(&mut self) {
        // SAFETY: the stored actions came from successful sigaction calls;
        // restore only our two dispositions while both saved values are live.
        unsafe {
            let _ = libc::sigaction(libc::SIGTERM, &self.previous_term, std::ptr::null_mut());
            let _ = libc::sigaction(libc::SIGINT, &self.previous_int, std::ptr::null_mut());
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub(super) struct Signals;

#[cfg(not(target_os = "linux"))]
impl Signals {
    pub(super) fn install() -> Result<Self, Failure> {
        Err(Failure::new("unsupported_platform"))
    }

    pub(super) fn requested(&self) -> bool {
        false
    }
}
