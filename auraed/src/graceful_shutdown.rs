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

use crate::runtime::CellService;
use tokio::signal::unix::SignalKind;
use tokio::sync::watch::{channel, Receiver, Sender};
use tracing::error;

pub(crate) struct GracefulShutdown {
    pub cell_service: CellService,
    shutdown_broadcaster: Sender<()>,
}

impl GracefulShutdown {
    pub fn new(cell_service: CellService) -> Self {
        let (tx, _) = channel(());
        Self { cell_service, shutdown_broadcaster: tx }
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
    pub async fn wait(self) {
        tokio::select! {
            _ = wait_for_sigterm() => {},
            _ = wait_for_sigint() => {},
        }

        self.shutdown_broadcaster.send_replace(());
        // wait for all subscribers to drop
        self.shutdown_broadcaster.closed().await;

        if let Err(e) = self.cell_service.free_all().await {
            error!(
                "Attempt to free all cells on terminate resulted in error: {e}"
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
