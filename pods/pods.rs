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

use super::{Pod, PodName, PodSpec, PodsError, Result};
use std::collections::HashMap;

type Cache = HashMap<PodName, Pod>;

/// The in-memory cache of pods ([Pod]) created with Aurae.
#[derive(Debug, Default)]
pub struct Pods {
    cache: Cache,
}

impl Pods {
    pub async fn allocate(
        &mut self,
        pod_name: PodName,
        pod_spec: PodSpec,
    ) -> Result<&Pod> {
        if self.cache.contains_key(&pod_name) {
            return Err(PodsError::PodExists { pod_name });
        }

        let pod = Pod::new(pod_name.clone(), pod_spec).await?;
        let pod = self.cache.entry(pod_name).or_insert(pod);

        pod.allocate().await?;

        Ok(pod)
    }

    pub fn free(&mut self, pod_name: &PodName) -> Result<()> {
        self.get_mut(pod_name, |pod| pod.free())?;
        let _ = self.cache.remove(pod_name);
        Ok(())
    }

    pub fn get<F, R>(&mut self, pod_name: &PodName, f: F) -> Result<R>
    where
        F: Fn(&Pod) -> Result<R>,
    {
        let Some(pod) = self.cache.get(pod_name) else {
            return Err(PodsError::PodNotFound { pod_name: pod_name.clone() });
        };

        let res = f(pod);

        if matches!(res, Err(PodsError::PodNotAllocated { .. })) {
            let _ = self.cache.remove(pod_name);
        }

        res
    }

    fn get_mut<F, R>(&mut self, pod_name: &PodName, f: F) -> Result<R>
    where
        F: FnOnce(&mut Pod) -> Result<R>,
    {
        let Some(pod) = self.cache.get_mut(pod_name) else {
            return Err(PodsError::PodNotFound { pod_name: pod_name.clone() });
        };

        let res = f(pod);

        if matches!(res, Err(PodsError::PodNotAllocated { .. })) {
            let _ = self.cache.remove(pod_name);
        }

        res
    }
}