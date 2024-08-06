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
/* -------------------------------------------------------------------------- *\
 *          Apache 2.0 License Copyright © 2022-2023 The Aurae Authors        *
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

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use x509_certificate::X509Certificate;

/// An in-memory representation of an X509 identity, and its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X509Details {
    /// From the SSL spec, the subject common name.
    pub subject_common_name: String,
    /// From the SSL spec, the issuer common name.
    pub issuer_common_name: String,
    /// From the SSL spec, the sha256 sum fingerprint of the material.
    pub sha256_fingerprint: String,
    /// From the SSL spec, the algorithm used for encryption.
    pub key_algorithm: String,
    // Force instantiation through function
    phantom_data: PhantomData<()>,
}

// This is purposefully not an associated function as instantiation of X509Details
// is being controlled in the module to limit the chance of misuse
pub(crate) fn new_x509_details(
    client_cert: Vec<u8>,
) -> anyhow::Result<X509Details> {
    let x509 = X509Certificate::from_pem(client_cert)?;

    let subject_common_name = x509.subject_common_name().ok_or_else(|| {
        anyhow!("Client certificated is missing subject_common_name")
    })?;

    let issuer_common_name = x509.issuer_common_name().ok_or_else(|| {
        anyhow!("Client certificate is missing issuer_common_name")
    })?;

    let sha256_fingerprint = x509.sha256_fingerprint()?;

    let key_algorithm = x509
        .key_algorithm()
        .ok_or_else(|| anyhow!("Client certificate is missing key_algorithm"))?
        .to_string();

    Ok(X509Details {
        subject_common_name,
        issuer_common_name,
        sha256_fingerprint: format!("{sha256_fingerprint:?}"),
        key_algorithm,
        phantom_data: PhantomData,
    })
}