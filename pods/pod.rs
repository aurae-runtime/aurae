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

use super::{PodName, PodSpec, Result};
use crate::runtime::pod_service::pods::PodsError;
use libcontainer::signal::Signal;
use libcontainer::{
    container::{builder::ContainerBuilder, Container},
    syscall::syscall::create_syscall,
};
use nix::sys::signal::{SIGKILL, SIGTERM};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Pod {
    name: PodName,
    spec: PodSpec,
    root_path: PathBuf,
    state: PodState,
}

#[derive(Debug)]
enum PodState {
    Unallocated,
    Allocated { container: Container },
    Freed,
}

impl Pod {
    pub async fn new(name: PodName, spec: PodSpec) -> Result<Self> {
        // TODO: do we need to be concerned with collisions from a nested auraed
        let root_path = PathBuf::from("/var/run/aurae/pods/{name}");

        if let Err(e) = tokio::fs::create_dir_all(&root_path).await {
            return Err(PodsError::FailedToCreateRootPathDirectory {
                pod_name: name,
                root_path,
                source: e,
            });
        }

        Ok(Self { name, spec, root_path, state: PodState::Unallocated })
    }

    /// Does nothing if [Pod] has been previously allocated.
    pub async fn allocate(&mut self) -> Result<()> {
        let PodState::Unallocated = &self.state else {
            return Ok(())
        };

        // NOTE (future-highway):
        //       The ocipkg crate seems to be working fine, but it doesn't have an
        //       async interface, which would probably be a good idea (using spawn_blocking for now).
        //       Going to continue with it as a dependency, and we can either, at some point,
        //       contribute to the crate or recreate the parts that we need in a local crate.
        //
        //       It also doesn't seem to let us control the location of the package on the
        //       filesystem, but I'm not sure if that is an OCI compliance choice. E.g.,
        //         - Image = index.docker.io/library/busybox:latest
        //         - Path  = /root/.local/share/ocipkg/index.docker.io/library/busybox/__latest

        // Download image and unpack
        let name = self.name.clone();
        let image = self.spec.image.clone();
        tokio::task::spawn_blocking(|| {
            // spawn_blocking does not automatically cancel or stop the thread,
            // so auraed shutdown will be delayed unless a workaround is used:
            // https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html

            match ocipkg::local::get_image_list() {
                Ok(images) => {
                    if !images.contains(&image) {
                        if let Err(e) = ocipkg::distribution::get_image(&image)
                        {
                            return Err(PodsError::FailedToPullImage {
                                pod_name: name,
                                image,
                                source: e,
                            });
                        }
                    }
                }
                Err(e) => {
                    return Err(PodsError::FailedToGetLocalImageList {
                        pod_name: name,
                        source: e,
                    })
                }
            };

            Ok::<_, PodsError>(())
        })
        .await??;

        // Location of unpacked image on the fs
        let image_path =
            ocipkg::local::image_dir(&self.spec.image).map_err(|e| {
                PodsError::FailedToFindLocalImage {
                    pod_name: self.name.clone(),
                    image: self.spec.image.clone(),
                    source: e,
                }
            })?;

        let container = ContainerBuilder::new(
            self.name.to_string(),
            create_syscall().as_ref(),
        )
        .with_root_path(self.root_path.clone())
        .map_err(|e| PodsError::FailedToSetContainerRootPath {
            pod_name: self.name.clone(),
            root_path: self.root_path.clone(),
            source: e,
        })?
        .as_init(image_path) // TODO (via @krisnova) This needs to be a lightweight "pause" container assembled at runtime from local data in the binary.
        .with_systemd(false) // defaults to true
        .build()
        .map_err(|e| PodsError::FailedToBuildContainer {
            pod_name: self.name.clone(),
            source: e,
        })?;

        self.state = PodState::Allocated { container };

        Ok(())
    }

    /// Signals the [Container] to gracefully shut down using [SIGTERM], and performs some cleaning up.
    /// The [Pod::state] will be set to [PodState::Freed] regardless of it's state prior to this call.
    /// A [Pod] should never be reused once in the [PodState::Freed] state.
    pub fn free(&mut self) -> Result<()> {
        self.do_free(|name, container| {
            // libcontainer uses an incompatible nix version (0.25.x) to the one we rely on (0.26.x),
            // preventing us from using `.into()`
            let signal =
                Signal::try_from(SIGTERM.as_str()).expect("valid signal");

            // Setting arg `all` to `true` allows for sending the signal to stopped containers.
            // Containers that are in the `Creating` state will error
            if let Err(e) = container.kill(signal, true) {
                return Err(PodsError::FailedToStopContainer {
                    pod_name: name.clone(),
                    source: e,
                });
            }

            Ok(())
        })
    }

    /// Sends a [SIGKILL] to the [Container], and performs some cleanup.
    /// The [Pod::state] will be set to [PodState::Freed] regardless of it's state prior to this call.
    /// A [Pod] should never be reused once in the [PodState::Freed] state.
    pub fn kill(&mut self) -> Result<()> {
        self.do_free(|name, container| {
            // libcontainer uses an incompatible nix version (0.25.x) to the one we rely on (0.26.x),
            // preventing us from using `.into()`
            let signal =
                Signal::try_from(SIGKILL.as_str()).expect("valid signal");

            // Setting arg `all` to `true` allows for sending the signal to stopped containers.
            // Containers that are in the `Creating` state will error
            if let Err(e) = container.kill(signal, true) {
                return Err(PodsError::FailedToKillContainer {
                    pod_name: name.clone(),
                    source: e,
                });
            }

            Ok(())
        })
    }

    fn do_free<F>(&mut self, f: F) -> Result<()>
    where
        F: Fn(&PodName, &mut Container) -> Result<()>,
    {
        if let PodState::Allocated { container } = &mut self.state {
            f(&self.name, container)?;

            std::fs::remove_dir_all(&self.root_path).map_err(|e| {
                PodsError::FailedToRemoveRootPathDirectory {
                    pod_name: self.name.clone(),
                    root_path: self.root_path.clone(),
                    source: e,
                }
            })?;
        }

        self.state = PodState::Freed;

        Ok(())
    }
}

impl Drop for Pod {
    fn drop(&mut self) {
        let _best_effort = self.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::{super::Image, *};
    use validation::ValidatedField;

    // TODO: update with a URL to an OCI Filesystem Bundle
    // Ignoring because this test fails since "index.docker.io/library/busybox:latest"
    // is not an OCI Filesystem Bundle
    #[ignore]
    #[tokio::test]
    async fn test() {
        let name = PodName::new("busybox".into());
        let spec = PodSpec {
            image: Image::validate(
                Some("index.docker.io/library/busybox:latest".into()),
                "test",
                None,
            )
            .expect("invalid image"),
        };
        let mut pod =
            Pod::new(name, spec).await.expect("failed to instantiate pod");
        pod.allocate().await.expect("failed to allocate pod");
    }
}