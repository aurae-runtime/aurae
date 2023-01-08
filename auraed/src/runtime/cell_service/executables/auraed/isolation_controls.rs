/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

use iter_tools::Itertools;
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::ptr;
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct IsolationControls {
    pub isolate_process: bool,
    pub isolate_network: bool,
}

#[derive(Default)]
pub(crate) struct Isolation {
    new_root: Option<PathBuf>,
}

impl Isolation {
    pub fn setup(&mut self, iso_ctl: &IsolationControls) -> io::Result<()> {
        // The only setup we will need to do is for isolate_process at this time.
        // We can exit quickly if we are sharing the process controls with the host.
        if !iso_ctl.isolate_process {
            return Ok(());
        }

        // Bind mount root:root with MS_REC and MS_PRIVATE flags
        // We are not sharing the mounts at this point (in other words we are in a new mount namespace)
        let root = PathBuf::from("/");
        nix::mount::mount(
            Some(&root),
            &root,
            None::<&str>, // ignored
            nix::mount::MsFlags::MS_PRIVATE | nix::mount::MsFlags::MS_REC,
            None::<&str>, // ignored
        )
        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
        info!("Isolation: Bind mounted root dir (/) in cell");
        Ok(())
    }

    pub fn isolate_process(
        &mut self,
        iso_ctl: &IsolationControls,
    ) -> io::Result<()> {
        if !iso_ctl.isolate_process {
            return Ok(());
        }

        //Mount proc in the new process
        let target = PathBuf::from("/proc");
        nix::mount::mount(
            Some("/proc"),
            &target,
            Some("proc"),
            nix::mount::MsFlags::empty(),
            None::<&str>,
        )
        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        info!("Isolate: Process");
        Ok(())
    }

    pub fn isolate_network(
        &mut self,
        iso_ctl: &IsolationControls,
    ) -> io::Result<()> {
        if !iso_ctl.isolate_network {
            return Ok(());
        }
        info!("Isolate: Network");
        Ok(())
    }

    pub fn mount_namespace_unmount_with_exceptions(
        &mut self,
    ) -> io::Result<()> {
        // NOTES: In this approach, we attempt to unmount all the mounts,
        //        but, unless we make exceptions (including '/') things break.

        // get a list of all the current mount points
        let mounts = procfs::process::Process::myself()
            .map_err(|e| io::Error::new(ErrorKind::Other, e))?
            .mountinfo()
            .map_err(|e| io::Error::new(ErrorKind::Other, e))?;

        // we are not sharing our mounts, so lets start by making all of them private
        // by making the root private and using MS_REC
        nix::mount::mount(
            None::<&str>, // ignored
            "/",
            None::<&str>, // ignored
            nix::mount::MsFlags::MS_PRIVATE | nix::mount::MsFlags::MS_REC,
            None::<&str>, // ignored
        )
        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        // now we want to unmount the mount points that were inherited from the parent
        // if a mount is below another mount, that has to be unmounted first
        for mount in mounts.iter().sorted_by_key(|x| x.mnt_id).rev() {
            // We skip...
            // - / -> things seem to break without it (but it seems wrong; not sure what to save)
            // - /run && /run/lock && /run/lock/1000 && /dev/shm ("tmpfs" types) -> same as "/"
            // - anything in new_root -> are we using it with this mount approach? I don't think so.
            if matches!(
                &*mount.fs_type,
                "tmpfs" | "procfs" //| "procfs" | "sysfs" | "devtmpfs" | "cgroup2"
            ) || matches!(&mount.mount_point, path if path.to_string_lossy() == "/" )
            {
                println!(
                    "Skipping mount point {:?} with type {:?}",
                    mount.mount_point, mount.fs_type
                );
                continue;
            }

            if let Some(new_root) = self.new_root.as_deref() {
                let new_root = new_root.to_string_lossy();
                if matches!(&mount.mount_point, path if path.to_string_lossy().starts_with(new_root.as_ref()))
                {
                    println!(
                        "Skipping mount point {:?} with type {:?}",
                        mount.mount_point, mount.fs_type
                    );
                    continue;
                }
            }

            println!(
                "Unmounting mount point {:?} with type {:?}",
                mount.mount_point, mount.fs_type
            );

            // now unmount it
            nix::mount::umount2(
                &mount.mount_point,
                nix::mount::MntFlags::MNT_DETACH,
            )
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
        }

        println!("mount namespace end");

        Ok(())
    }
    //
    // pub fn mount_namespace_pivot_root(
    //     &mut self,
    //     iso_ctl: &IsolationControls,
    // ) -> io::Result<()> {
    //     // NOTES: In this approach, we create a new clean root using pivot_root,
    //     //        which seems like the correct and safe approach.
    //     //        But, while pre_exec completes (unshared mount + pid), we get
    //     //        "No such file or directory (os error 2)" seemingly at the call that
    //     //        std's Command makes to libc::execvp.
    //     //
    //     //        I did not try mounting the parent root onto the new root to mimic
    //     //        saving "/" like I did in the other (mount_namespace_unmount_with_exceptions)
    //     //        approach.
    //
    //     if shared_namespaces.mount {
    //         // see note in other approach
    //         return Ok(());
    //     }
    //
    //     println!("mount namespace start -- pivot root");
    //
    //     // we are not sharing our mounts, so lets start by making all of them private
    //     // by making the root private and using MS_REC
    //     nix::mount::mount(
    //         None::<&str>, // ignored
    //         "/",
    //         None::<&str>, // ignored
    //         nix::mount::MsFlags::MS_PRIVATE | nix::mount::MsFlags::MS_REC,
    //         None::<&str>, // ignored
    //     )
    //     .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
    //
    //     let new_root = self.new_root.as_ref().expect("didn't call prep");
    //
    //     nix::unistd::chdir(new_root)
    //         .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
    //
    //     println!("changed into the soon to be new root dir");
    //
    //     nix::unistd::pivot_root(".", ".")
    //         .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
    //
    //     println!("pivoted the old root to below the new root");
    //
    //     nix::mount::umount2(".", nix::mount::MntFlags::MNT_DETACH)
    //         .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
    //
    //     println!("unmounted old root");
    //
    //     nix::unistd::chdir("/")
    //         .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
    //
    //     println!("changed to new root directory");
    //
    //     println!("mount namespace done");
    //
    //     Ok(())
    // }
}
