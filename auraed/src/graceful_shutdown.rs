/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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

use crate::{cells::CellService, discovery::DiscoveryService};
use proto::{
    cells::cell_service_server::CellServiceServer,
    discovery::discovery_service_server::DiscoveryServiceServer,
};
use std::borrow::BorrowMut;
use tokio::{
    signal::unix::SignalKind,
    sync::watch::{channel, Receiver, Sender},
};
use tonic_health::server::HealthReporter;
use tracing::error;

pub(crate) struct GracefulShutdown {
    health_reporter: HealthReporter,
    cell_service: CellService,
    shutdown_broadcaster: Sender<()>,
}

impl GracefulShutdown {
    pub fn new(
        health_reporter: HealthReporter,
        cell_service: CellService,
    ) -> Self {
        let (tx, _) = channel(());
        Self { health_reporter, cell_service, shutdown_broadcaster: tx }
    }

    /// Subscribe to the shutdown broadcast channel
    pub fn subscribe(&self) -> Receiver<()> {
        self.shutdown_broadcaster.subscribe()
    }

    /// Waits for a signal and then...
    /// * Broadcasts a shutdown signal to all subscribers. See [subscribe]
    /// * Waits for all subscribers to drop
    /// * Calls [CellService::free_all]
    /// ---
    /// Signals:
    /// * [SIGTERM]
    /// * [SIGINT]
    /// ---
    /// Returns after processing the first received signal.
    pub async fn wait(mut self) {
        tokio::select! {
            _ = wait_for_sigterm() => {},
            _ = wait_for_sigint() => {},
        }

        // update health reporter
        let health_reporter = self.health_reporter.borrow_mut();
        health_reporter
            .set_not_serving::<CellServiceServer<CellService>>()
            .await;
        health_reporter
            .set_not_serving::<DiscoveryServiceServer<DiscoveryService>>()
            .await;

        // health_reporter.set_not_serving::<PodServiceServer<PodService>>().await;

        self.shutdown_broadcaster.send_replace(());
        // wait for all subscribers to drop
        self.shutdown_broadcaster.closed().await;

        if let Err(e) = self.cell_service.free_all().await {
            error!(
                "Attempt to free all cells on terminate resulted in error: {e}"
            )
        }

        if let Err(e) = self.cell_service.stop_all().await {
            error!(
                "Attempt to stop all executables on terminate resulted in error: {e}"
            )
        }
    }
}

pub async fn wait_for_sigterm() {
    let mut stream = tokio::signal::unix::signal(SignalKind::terminate())
        .expect("failed to listen for SIGTERM");

    let _ = stream.recv().await;
}

pub async fn wait_for_sigint() {
    let mut stream = tokio::signal::unix::signal(SignalKind::interrupt())
        .expect("failed to listen for SIGINT");

    let _ = stream.recv().await;
}
