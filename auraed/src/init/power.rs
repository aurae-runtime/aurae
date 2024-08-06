/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */

use anyhow::anyhow;
use std::{fs::OpenOptions, io::Read, mem, path::Path, slice};
use tracing::{info, trace};

use ::libc;

pub(crate) fn syscall_reboot(action: i32) {
    unsafe {
        if libc::reboot(action) != 0 {
            // TODO: handle this better
            panic!("failed to reboot");
        }
    }
}

pub(crate) fn power_off() {
    syscall_reboot(libc::LINUX_REBOOT_CMD_POWER_OFF);
}

pub(crate) fn reboot() {
    syscall_reboot(libc::LINUX_REBOOT_CMD_RESTART);
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(C, packed)]
pub(crate) struct InputEvent {
    tv_sec: u64,
    tv_usec: u64,
    evtype: u16,
    code: u16,
    value: u32,
}

// see  https://elixir.bootlin.com/linux/latest/source/include/uapi/linux/input-event-codes.h#L191
const KEY_POWER: u16 = 116;
const KEY_RESTART: u16 = 0x198;

fn to_u8_ptr<T>(p: *mut T) -> *mut u8 {
    p as _
}

pub(crate) fn spawn_thread_power_button_listener(
    power_btn_device_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let mut event_file = match OpenOptions::new()
        .read(true)
        .write(false)
        .open(&power_btn_device_path)
    {
        Ok(file) => file,
        Err(e) => {
            return Err(anyhow!(
                "Could not open power button device {}. {:?}",
                power_btn_device_path.as_ref().display(),
                e
            ));
        }
    };

    let mut event: InputEvent = unsafe { mem::zeroed() };
    let event_size = mem::size_of::<InputEvent>();

    let power_btn_device = power_btn_device_path.as_ref().to_owned();
    let _ = std::thread::spawn(move || {
        loop {
            let event_slice = unsafe {
                slice::from_raw_parts_mut(to_u8_ptr(&mut event), event_size)
            };
            match event_file.read(event_slice) {
                Ok(result) => {
                    trace!("Event0: {} {:?}", result, event);
                    if event.code == KEY_POWER {
                        // TODO: shutdown runtime
                        // - need to send signal via a channel to runtime
                        // - await for runtime
                        info!("Power Button pressed - shutting down now");
                        power_off();
                    } else if event.code == KEY_RESTART {
                        info!("Restart Button pressed - rebooting now");
                        reboot();
                    }
                }
                Err(e) => {
                    return Err::<(), anyhow::Error>(anyhow!(
                        "Could not parse event from {}: {:?}",
                        power_btn_device.display(),
                        e
                    ));
                }
            }
        }
    });
    Ok(())
}