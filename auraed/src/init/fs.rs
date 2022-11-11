use log::{error, info};
use std::ffi::CStr;
use std::{io, ptr};

#[derive(thiserror::Error, Debug)]
pub(crate) enum FsError {
    #[error("Failed to mount (source_name={source_name}, target_name={target_name}, fstype={fstype}) due to error: {source}")]
    MountFailure {
        source_name: String,
        target_name: String,
        fstype: String,
        source: io::Error,
    },
}

pub(crate) fn mount_vfs(
    source_name: &CStr,
    target_name: &CStr,
    fstype: &CStr,
) -> Result<(), FsError> {
    info!("Mounting {:?}", target_name);

    let ret = {
        #[cfg(not(target_os = "macos"))]
        unsafe {
            libc::mount(
                source_name.as_ptr(),
                target_name.as_ptr(),
                fstype.as_ptr(),
                0,
                ptr::null(),
            )
        }

        #[cfg(target_os = "macos")]
        unsafe {
            libc::mount(
                src_c_str.as_ptr(),
                target_name_c_str.as_ptr(),
                0,
                ptr::null_mut(),
            )
        }
    };

    if ret < 0 {
        let error = io::Error::last_os_error();
        error!("Failed to mount ({})", error);

        Err(FsError::MountFailure {
            source_name: source_name.to_string_lossy().to_string(),
            target_name: target_name.to_string_lossy().to_string(),
            fstype: fstype.to_string_lossy().to_string(),
            source: error,
        })
    } else {
        Ok(())
    }
}
