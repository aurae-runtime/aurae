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
pub struct SharedNamespaces {
    pub mount: bool,
    pub pid: bool,
    pub net: bool,
}

#[derive(Default)]
pub(crate) struct Unshare {
    new_root: Option<PathBuf>,
}

impl Unshare {
    pub fn prep(
        &mut self,
        shared_namespaces: &SharedNamespaces,
    ) -> io::Result<()> {
        // This is run in the parent, prior to the call to clone3.
        //
        // The rest of the functions are called in pre_exec, after clone, and in the following order:
        //   pid, mount, network.
        //
        // Ideally, order won't matter, or the code will be refactored so that it can
        // only be called correctly
        //
        // So far, we need a new root if
        // * mount is not shared
        // * mount is shared, but pid is not shared
        //
        // However, I suspect we may always want a new root, followed by adding what
        // we want into that new root.
        //
        // These conditionals are written weirdly so that they align with the points above
        #[allow(clippy::nonminimal_bool)]
        if !(!shared_namespaces.mount)
            && !(shared_namespaces.mount && !shared_namespaces.pid)
        {
            return Ok(());
        }

        let new_root = PathBuf::from(format!("/ae-{}", uuid::Uuid::new_v4()));

        // As long as these are in prep, calls to std are ok. Otherwise, need to use libc and nix.
        std::fs::create_dir(&new_root)?;

        // NOTES: We are copying auraed into the new root in an attempt to fix the
        //        "No such file or directory (os error 2) that we will eventually get.
        //        This change does not fix it and while I'm not 100% sure that the missing
        //        file is referring to auraed, I feel like it only makes sense that auraed
        //        be present in the new mount space
        let _ =
            std::fs::copy("/root/.cargo/bin/auraed", new_root.join("auraed"))?;

        // we need a directory for each mount point we intend to make
        for dir in ["dev", "sys", "proc", "bin"] {
            let dir = new_root.join(dir);
            std::fs::create_dir_all(&dir)?;
            println!("created {dir:?} dir under new root dir");
        }

        // make the new root a mount point
        nix::mount::mount(
            Some(&new_root),
            &new_root,
            None::<&str>, // ignored,
            nix::mount::MsFlags::MS_BIND,
            None::<&str>, // ignored
        )
        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        println!("bind mounted new root dir (rec)");

        // TODO: the vfs and other directory mounts seemingly get doubled (why?)

        // mount the vfs directories (same virtual filesystems that are in the init module)
        for dir in [
            (None, "dev", "devtmpfs"),
            (None, "sys", "sysfs"),
            (Some("proc"), "proc", "proc"),
        ] {
            let (source, target, fstype) = dir;
            let target = &new_root.join(target);

            nix::mount::mount(
                source,
                target,
                Some(fstype),
                nix::mount::MsFlags::empty(),
                None::<&str>,
            )
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

            println!("mounted {target:?}");
        }

        // NOTE: including bin was an attempt at solving an error where I inserted
        //       a call to `ls` at the end of pre_exec (when changing the root) to try
        //       and see the state of the system. It didn't work, so is just left
        //       as an example/placeholder (aka, I got tired of deleting and rewriting
        //       seemingly the same code).

        // mount other directories
        #[allow(clippy::single_element_loop)]
        for dir in [(Some("/bin"), "bin")] {
            let (source, target) = dir;
            let target = &new_root.join(target);

            nix::mount::mount(
                source,
                target,
                None::<&str>, // ignored,
                nix::mount::MsFlags::MS_BIND | nix::mount::MsFlags::MS_REC,
                None::<&str>, // ignored
            )
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

            println!("mounted {target:?}");
        }

        self.new_root = Some(new_root);

        Ok(())
    }

    pub fn mount_namespace_unmount_with_exceptions(
        &mut self,
        shared_namespaces: &SharedNamespaces,
    ) -> io::Result<()> {
        // NOTES: In this approach, we attempt to unmount all the mounts,
        //        but, unless we make exceptions (including '/') things break.

        if shared_namespaces.mount {
            // Do we want to chroot and remount everything in the new root?
            // what would that do to the parent?
            //
            // Also, if we do use chroot, it does seem to be considered "cosmetic", for
            // lack of a better vocabulary at this moment :), with pivot root being preferred
        } else {
            println!("mount namespace start -- unmount");

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
                // - /proc -> this is handled by pid_namespace
                // - / -> things seem to break without it (but it seems wrong; not sure what to save)
                // - /run && /run/lock && /run/lock/1000 && /dev/shm ("tmpfs" types) -> same as "/"
                // - anything in new_root -> are we using it with this mount approach? I don't think so.
                if matches!(
                    &*mount.fs_type,
                    "proc" | "tmpfs" //| "sysfs" | "devtmpfs" | "cgroup2"
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
        }

        Ok(())
    }

    pub fn mount_namespace_pivot_root(
        &mut self,
        shared_namespaces: &SharedNamespaces,
    ) -> io::Result<()> {
        // NOTES: In this approach, we create a new clean root using pivot_root,
        //        which seems like the correct and safe approach.
        //        But, while pre_exec completes (unshared mount + pid), we get
        //        "No such file or directory (os error 2)" seemingly at the call that
        //        std's Command makes to libc::execvp.
        //
        //        I did not try mounting the parent root onto the new root to mimic
        //        saving "/" like I did in the other (mount_namespace_unmount_with_exceptions)
        //        approach.

        if shared_namespaces.mount {
            // see note in other approach
            return Ok(());
        }

        println!("mount namespace start -- pivot root");

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

        let new_root = self.new_root.as_ref().expect("didn't call prep");

        nix::unistd::chdir(new_root)
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        println!("changed into the soon to be new root dir");

        nix::unistd::pivot_root(".", ".")
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        println!("pivoted the old root to below the new root");

        nix::mount::umount2(".", nix::mount::MntFlags::MNT_DETACH)
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        println!("unmounted old root");

        nix::unistd::chdir("/")
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        println!("changed to new root directory");

        println!("mount namespace done");

        Ok(())
    }

    pub fn pid_namespace(
        &mut self,
        shared_namespaces: &SharedNamespaces,
    ) -> io::Result<()> {
        if shared_namespaces.pid {
            return Ok(());
        }

        // NOTES: [Docs](https://man7.org/linux/man-pages/man7/pid_namespaces.7.html)
        //        say that child processes, should either:
        //        A) when mount is shared, change the root of the child and mount /proc
        //           under the new root, or
        //        B) when mount is not shared, changing root is not needed, just mount over /proc
        //
        //        Changing the root causes issues. Probably the same as when we
        //        pivot_root or unmount "/"

        let target = if shared_namespaces.mount {
            let new_root = self.new_root.as_deref().expect("didn't call prep");
            new_root.join("proc")
        } else {
            PathBuf::from("/proc")
        };

        // mount over the parent mount
        // TODO: check that an umount /proc doesn't expose the parent mount
        nix::mount::mount(
            Some("/proc"),
            &target,
            Some("proc"),
            nix::mount::MsFlags::empty(),
            None::<&str>,
        )
        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        // NOTE: The docs say to change the root, but if we do, it seems the auraed
        //       exe cannot be found anymore. However, since we don't, we are seeing the
        //       parent's version of /proc.
        if shared_namespaces.mount {
            // let new_root = self.new_root.as_deref().expect("didn't call prep");
            // nix::unistd::chroot(new_root)
            //     .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
        }
        info!("Unshare: New pid namespace");
        Ok(())
    }

    pub fn net_namespace(
        &mut self,
        shared_namespaces: &SharedNamespaces,
    ) -> io::Result<()> {
        if shared_namespaces.net {
            return Ok(());
        }
        info!("Unshare: New net namespace");
        Ok(())
    }
}
