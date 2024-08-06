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

use crate::runtime::pod_service::pods::image::Image;
use crate::runtime::pod_service::pods::pod_name::PodName;
use std::{io, path::PathBuf};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PodsError>;

#[derive(Error, Debug)]
pub enum PodsError {
    #[error("pod '{pod_name}' already exists'")]
    PodExists { pod_name: PodName },
    #[error("pod '{pod_name}' not found")]
    PodNotFound { pod_name: PodName },
    #[error("pod '{pod_name}' is not allocated")]
    PodNotAllocated { pod_name: PodName },
    #[error(
        "pod '{pod_name}' failed to create directory '{root_path}': {source}"
    )]
    FailedToCreateRootPathDirectory {
        pod_name: PodName,
        root_path: PathBuf,
        source: io::Error,
    },
    #[error(
        "pod '{pod_name}' failed to remove directory '{root_path}': {source}"
    )]
    FailedToRemoveRootPathDirectory {
        pod_name: PodName,
        root_path: PathBuf,
        source: io::Error,
    },
    #[error("pod '{pod_name}' failed to get local image list: {source}")]
    FailedToGetLocalImageList {
        pod_name: PodName,
        source: ocipkg::error::Error,
    },
    #[error("pod '{pod_name}' failed to pull image '{image}': {source}")]
    FailedToPullImage {
        pod_name: PodName,
        image: Image,
        source: ocipkg::error::Error,
    },
    #[error("pod '{pod_name}' failed to find local image '{image}': {source}")]
    FailedToFindLocalImage {
        pod_name: PodName,
        image: Image,
        source: ocipkg::error::Error,
    },
    #[error("pod '{pod_name}' failed to set directory '{root_path}' as container root path: {source}")]
    FailedToSetContainerRootPath {
        pod_name: PodName,
        root_path: PathBuf,
        source: anyhow::Error,
    },
    #[error("pod '{pod_name}' failed to build container: {source}")]
    FailedToBuildContainer { pod_name: PodName, source: anyhow::Error },
    #[error("pod '{pod_name}' failed to stop container: {source}")]
    FailedToStopContainer { pod_name: PodName, source: anyhow::Error },
    #[error("pod '{pod_name}' failed to kill container: {source}")]
    FailedToKillContainer { pod_name: PodName, source: anyhow::Error },
    #[error(transparent)]
    TaskJoinError(#[from] tokio::task::JoinError),
}