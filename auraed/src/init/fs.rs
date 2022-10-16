use log::{error, info};
use std::ffi::{CString, NulError};
use std::ptr;

#[derive(thiserror::Error, Debug)]
pub(crate) enum FsError {
    #[error("{source_name} cannot be converted to a CString")]
    InvalidSourceName { source_name: String, source: NulError },
    #[error("{target_name} cannot be converted to a CString")]
    InvalidTargetName { target_name: String, source: NulError },
    #[error("{fstype} cannot be converted to a CString")]
    InvalidFstype { fstype: String, source: NulError },
    #[error("Failed to mount: source_name={source_name}, target_name={target_name}, fstype={fstype}")]
    MountFailure { source_name: String, target_name: String, fstype: String },
}

pub(crate) fn mount_vfs(
    source_name: &str,
    target_name: &str,
    fstype: &str,
) -> Result<(), FsError> {
    info!("Mounting {}", target_name);

    // CString constructor ensures the trailing 0byte, which is required by libc::mount
    let src_c_str =
        CString::new(source_name).map_err(|e| FsError::InvalidSourceName {
            source_name: source_name.to_owned(),
            source: e,
        })?;

    let target_name_c_str =
        CString::new(target_name).map_err(|e| FsError::InvalidTargetName {
            target_name: target_name.to_owned(),
            source: e,
        })?;

    let ret = {
        #[cfg(not(target_os = "macos"))]
        {
            let fstype_c_str = CString::new(fstype).map_err(|e| {
                FsError::InvalidFstype { fstype: fstype.to_owned(), source: e }
            })?;

            unsafe {
                libc::mount(
                    src_c_str.as_ptr(),
                    target_name_c_str.as_ptr(),
                    fstype_c_str.as_ptr(),
                    0,
                    ptr::null(),
                )
            }
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
            source_name: source_name.to_owned(),
            target_name: target_name.to_owned(),
            fstype: fstype.to_owned(),
        })
    } else {
        Ok(())
    }
}
