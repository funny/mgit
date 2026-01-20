#[cfg(windows)]
mod windows {
    use std::mem;
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    struct JobHandle(HANDLE);
    unsafe impl Send for JobHandle {}
    unsafe impl Sync for JobHandle {}

    // Global Job Object handle
    static JOB_OBJECT: OnceLock<JobHandle> = OnceLock::new();

    unsafe fn create_job_object() -> HANDLE {
        let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job == 0 as _ {
            panic!("Failed to create job object");
        }

        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        let r = SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );

        if r == 0 {
            panic!("Failed to set job object information");
        }

        job
    }

    pub fn get_global_job_object() -> HANDLE {
        JOB_OBJECT
            .get_or_init(|| unsafe { JobHandle(create_job_object()) })
            .0
    }

    pub fn assign_process_to_job(process_handle: HANDLE) -> bool {
        let job = get_global_job_object();
        unsafe { AssignProcessToJobObject(job, process_handle) != 0 }
    }
}

#[cfg(not(windows))]
mod unix {
    // For Unix, we rely on tokio::process::Command behavior or pdeathsig if implemented
    // Currently tokio handles child cleanup reasonably well, but for stricter control
    // we might need nix crate. For now, we leave it simple.
    pub fn assign_process_to_job<T>(_process: &T) -> bool {
        true
    }
}

use tokio::process::Command;

pub struct ProcessGuard {
    // We don't need to hold the Child process here if we are just spawning it.
    // But if we want to ensure cleanup, we might wrap Child.
    // For now, this module provides the helper to attach the process.
}

impl ProcessGuard {
    /// Configure the command to be part of the job object (on Windows)
    pub fn configure(_cmd: &mut Command) {
        #[cfg(windows)]
        {
            // Windows Job Object assignment happens AFTER spawn, but we can't easily hook into tokio::process::Command spawn
            // unless we wrap the Child.
            // Wait, AssignProcessToJobObject takes a process handle.
            // We need to call it on the Child's raw handle.
        }
    }

    /// Attach a spawned child to the global job object
    pub fn attach(child: &tokio::process::Child) {
        #[cfg(windows)]
        {
            if let Some(raw_handle) = child.raw_handle() {
                // AsRawHandle returns RawHandle which is compatible with HANDLE (void*)
                // windows-sys HANDLE is isize/usize depending on arch, usually *mut c_void
                let handle = raw_handle as windows_sys::Win32::Foundation::HANDLE;
                windows::assign_process_to_job(handle);
            }
        }
        #[cfg(not(windows))]
        {
            let _ = child;
        }
    }
}
