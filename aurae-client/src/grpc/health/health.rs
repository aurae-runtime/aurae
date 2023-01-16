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

// TODO: The macro does not support streaming, see below for what we want the macro to output
// macros::service!(
//     grpc::health,
//     Health,
//     check(HealthCheckRequest) -> HealthCheckResponse,
//     watch(HealthCheckRequest) -> HealthCheckResponse
// );

#[::tonic::async_trait]
pub trait HealthClient {
    async fn check(
        &self,
        req: ::aurae_proto::grpc::health::HealthCheckRequest,
    ) -> Result<
        ::tonic::Response<::aurae_proto::grpc::health::HealthCheckResponse>,
        ::tonic::Status,
    >;
    async fn watch(
        &self,
        req: ::aurae_proto::grpc::health::HealthCheckRequest,
    ) -> Result<
        ::tonic::Response<
            ::tonic::Streaming<
                ::aurae_proto::grpc::health::HealthCheckResponse,
            >,
        >,
        ::tonic::Status,
    >;
}
#[::tonic::async_trait]
impl HealthClient for crate::client::AuraeClient {
    async fn check(
        &self,
        req: ::aurae_proto::grpc::health::HealthCheckRequest,
    ) -> Result<
        ::tonic::Response<::aurae_proto::grpc::health::HealthCheckResponse>,
        ::tonic::Status,
    > {
        let mut client =
            ::aurae_proto::grpc::health::health_client::HealthClient::new(
                self.channel.clone(),
            );
        client.check(req).await
    }

    async fn watch(
        &self,
        req: ::aurae_proto::grpc::health::HealthCheckRequest,
    ) -> Result<
        ::tonic::Response<
            ::tonic::Streaming<
                ::aurae_proto::grpc::health::HealthCheckResponse,
            >,
        >,
        ::tonic::Status,
    > {
        let mut client =
            ::aurae_proto::grpc::health::health_client::HealthClient::new(
                self.channel.clone(),
            );
        client.watch(req).await
    }
}

