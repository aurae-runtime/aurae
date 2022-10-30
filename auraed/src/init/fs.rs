use log::{error, info};
use std::ffi::{CStr, CString};
use std::ptr;

#[derive(thiserror::Error, Debug)]
pub(crate) enum FsError {
    #[error("Failed to mount: source_name={source_name}, target_name={target_name}, fstype={fstype}")]
    MountFailure { source_name: String, target_name: String, fstype: String },
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
        error!("Failed to mount ({})", ret);
        let error = CString::new("Error: ").expect("error creating CString");
        unsafe {
            libc::perror(error.as_ptr());
        };

        Err(FsError::MountFailure {
            source_name: source_name.to_string_lossy().to_string(),
            target_name: target_name.to_string_lossy().to_string(),
            fstype: fstype.to_string_lossy().to_string(),
        })
    } else {
        Ok(())
    }
}
