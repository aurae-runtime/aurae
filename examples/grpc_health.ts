#!/usr/bin/env auraescript
/* -------------------------------------------------------------------------- *\
 *               Apache 2.0 License Copyright The Aurae Authors               *
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

import * as helpers from "../auraescript/gen/helpers.ts";
import * as grpc_health from "../auraescript/gen/grpc_health.ts";

let healthClient = new grpc_health.HealthClient;

// [ Grpc health ]
helpers.print("Checking overall status")
let overall = await healthClient.check(<grpc_health.HealthCheckRequest>{
    service: ""
});
helpers.print(overall)

helpers.print("Checking CellService status")
let single_service = await healthClient.check(<grpc_health.HealthCheckRequest>{
    service: "aurae.runtime.v0.CellService"
});
helpers.print(single_service)

helpers.print("Checking status of unregistered service")
let unknown_service = await healthClient.check(<grpc_health.HealthCheckRequest>{
    service: "aurae.runtime.v0.UnknownService"
});
helpers.print(unknown_service)
