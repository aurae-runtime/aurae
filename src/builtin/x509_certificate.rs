use anyhow::{anyhow, Result};
use if_chain::if_chain;
use oid_registry::Oid;
use time::OffsetDateTime;
use x509_parser::{
    certificate::X509Certificate as X509CertificateParser,
    prelude::FromDer,
    x509::{X509Name, X509Version},
};

// TODO: X509Certificate, see comments
#[allow(dead_code)] // Will be used; suppress warnings for now
#[derive(Debug)]
pub(crate) struct X509Certificate {
    pub version: Version,
    /// Big-endian
    pub serial_number: Vec<u8>,
    pub signature_algorithm: SignatureAlgorithm,
    pub issuer: DistinguishedName,
    pub validity_not_before: OffsetDateTime,
    pub validity_not_after: OffsetDateTime,
    pub subject: DistinguishedName,
    pub subject_alternative_names: Vec<Vec<u8>>, // there can be more than 1 of these, right?, also the encoding is tripping me up. Seems like it depends on the value. What types of values should be expected/supported?
}

impl X509Certificate {
    pub fn from_pem(cert_material: &[u8]) -> Result<Self> {
        let mut cert = std::io::Cursor::new(cert_material);

        let cert = if_chain! {
            if let Ok(certs) = rustls_pemfile::certs(&mut cert);
            if let Some(cert) = certs.first().as_ref();
            if let Ok((_rem, cert)) = X509CertificateParser::from_der(cert);
            then {
                let version = Version::from(cert.version);
                let serial_number = cert.serial.to_bytes_be();
                let signature_algorithm = SignatureAlgorithm::from(&cert.signature_algorithm.algorithm);
                let issuer = DistinguishedName::try_from(&cert.issuer)?;
                let validity_not_before = cert.validity.not_before.to_datetime();
                let validity_not_after = cert.validity.not_after.to_datetime();
                let subject = DistinguishedName::try_from(&cert.subject)?;
                let subject_alternative_names: Vec<Vec<u8>> = cert.extensions().iter().filter_map(|extension| {
                    // there is also OID_X509_OBSOLETE_SUBJECT_ALT_NAME
                    if extension.oid == oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME {
                        Some(extension.value.to_vec())
                    } else {
                        None
                    }
                }).collect();

                Self {
                    version,
                    serial_number,
                    signature_algorithm,
                    issuer,
                    validity_not_before,
                    validity_not_after,
                    subject,
                    subject_alternative_names
                }
            } else {
                return Err(anyhow!("unable to parse x509"));
            }
        };

        Ok(cert)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Version {
    V1,
    V2,
    V3,
    // versions seems to be 0 indexed (i.e., V1 = 0), so a V4 would be Other(3),
    // which seems confusing. Is there a better way?
    Other(u32),
}

impl From<X509Version> for Version {
    fn from(version: X509Version) -> Self {
        match version {
            X509Version::V1 => Self::V1,
            X509Version::V2 => Self::V2,
            X509Version::V3 => Self::V3,
            X509Version(version) => Self::Other(version),
        }
    }
}

#[derive(Default, Debug, Eq, PartialEq)]
pub(crate) enum SignatureAlgorithm {
    #[default]
    Unsupported,
    Sha256WithRSAEncryption,
    // what other algos do we expect/support
}

impl<'a> From<&Oid<'a>> for SignatureAlgorithm {
    fn from(oid: &Oid<'_>) -> Self {
        if *oid == oid_registry::OID_PKCS1_SHA256WITHRSA {
            Self::Sha256WithRSAEncryption
        } else {
            Self::Unsupported
        }
    }
}

// Are these fields guaranteed to be utf8 (else cert is invalid)?
//      x509_parser crate makes considerations for non utf8 in a more generally used fn.
// Seems like they can all technically have multiple values.
// RFC: https://datatracker.ietf.org/doc/html/rfc5280#section-4.1.2.4
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct DistinguishedName {
    pub countries: Vec<String>,
    pub states_or_provinces: Vec<String>,
    pub localities: Vec<String>,
    pub organizations: Vec<String>,
    pub organization_units: Vec<String>, // deprecated?
    pub common_names: Vec<String>,
}

impl<'a> TryFrom<&X509Name<'a>> for DistinguishedName {
    type Error = anyhow::Error;

    fn try_from(value: &X509Name) -> std::result::Result<Self, Self::Error> {
        // This can be more performant using oid_registry and cycling through the iter 1x
        //
        // While these attributes may be valid utf8 (I haven't confirmed),
        //  the rfc indicates that the encoding is OID dependant,
        //  so String will not necessarily be appropriate for all the data ultimately wanted.

        let mut countries: Vec<String> = vec![];
        for country in value.iter_country() {
            countries.push(country.as_str()?.to_owned())
        }

        let mut states_or_provinces: Vec<String> = vec![];
        for state_or_province in value.iter_state_or_province() {
            states_or_provinces.push(state_or_province.as_str()?.to_owned())
        }

        let mut localities: Vec<String> = vec![];
        for locality in value.iter_locality() {
            localities.push(locality.as_str()?.to_owned())
        }

        let mut organizations: Vec<String> = vec![];
        for organization in value.iter_organization() {
            organizations.push(organization.as_str()?.to_owned())
        }

        let mut organization_units: Vec<String> = vec![];
        for organization_unit in value.iter_organizational_unit() {
            organization_units.push(organization_unit.as_str()?.to_owned())
        }

        let mut common_names: Vec<String> = vec![];
        for common_name in value.iter_common_name() {
            common_names.push(common_name.as_str()?.to_owned())
        }

        Ok(Self {
            countries,
            states_or_provinces,
            localities,
            organizations,
            organization_units,
            common_names,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openssl::asn1::Asn1Integer;
    use openssl::{
        asn1::Asn1Time,
        bn::{BigNum, MsbOption},
        hash::MessageDigest,
        pkey::PKey,
        rsa::Rsa,
        stack::Stack,
        x509::{
            extension::{
                AuthorityKeyIdentifier, BasicConstraints, KeyUsage,
                SubjectAlternativeName,
            },
            X509Name, X509NameBuilder, X509ReqBuilder, X509,
        },
    };
    use std::ops::{Add, Sub};
    use time::Duration;

    const DISTINGUISHED_NAME_C: &str = "IS";
    const DISTINGUISHED_NAME_ST: &str = "aurae";
    const DISTINGUISHED_NAME_L: &str = "aurae";
    const DISTINGUISHED_NAME_O: &str = "Aurae";
    const DISTINGUISHED_NAME_OU: &str = "Runtime";
    const NOT_BEFORE_DAYS: u32 = 2000;
    const DAYS_VALID: u32 = 1;
    const CA_DISTINGUISHED_NAME_CN: &str = "test.material";
    const CA_SUBJECT_ALT_NAME: &str = "test.material";
    const CLIENT_DISTINGUISHED_NAME_CN: &str = "example.test.material";
    const CLIENT_SUBJECT_ALT_NAME: &str = "example.test.material";

    struct TestCerts {
        ca_crt: Vec<u8>,
        ca_serial_number: Vec<u8>,
        client_crt_signed: Vec<u8>,
        client_crt_signed_serial_number: Vec<u8>,
    }

    impl TestCerts {
        fn new() -> Result<Self> {
            let (ca_cert, ca_crt_pem, ca_key_pair, ca_serial_number) = {
                let rsa = Rsa::generate(4096)?;
                let key_pair = PKey::from_rsa(rsa)?;
                let subject = build_subject_name(CA_DISTINGUISHED_NAME_CN)?;

                let mut cert_builder = X509::builder()?;
                cert_builder.set_version(2)?;

                let (serial_number, serial_number_bytes) =
                    generate_serial_number()?;
                cert_builder.set_serial_number(&serial_number)?;

                cert_builder.set_subject_name(&subject)?;
                cert_builder.set_issuer_name(&subject)?;
                cert_builder.set_pubkey(&key_pair)?;

                let not_before = Asn1Time::days_from_now(NOT_BEFORE_DAYS)?;
                cert_builder.set_not_before(&not_before)?;

                let not_after =
                    Asn1Time::days_from_now(NOT_BEFORE_DAYS + DAYS_VALID)?;
                cert_builder.set_not_after(&not_after)?;

                cert_builder.append_extension(
                    SubjectAlternativeName::new()
                        .dns(CA_SUBJECT_ALT_NAME)
                        .build(&cert_builder.x509v3_context(None, None))?,
                )?;

                cert_builder.sign(&key_pair, MessageDigest::sha256())?;
                let crt = cert_builder.build();
                let pem = crt.to_pem()?;
                (crt, pem, key_pair, serial_number_bytes)
            };

            let (client_csr, client_key_pair) = {
                let rsa = Rsa::generate(4096)?;
                let key_pair = PKey::from_rsa(rsa)?;

                let mut req_builder = X509ReqBuilder::new()?;
                req_builder.set_pubkey(&key_pair)?;

                let subject = build_subject_name(CLIENT_DISTINGUISHED_NAME_CN)?;
                req_builder.set_subject_name(&subject)?;

                let mut extensions = Stack::new()?;
                extensions.push(
                    SubjectAlternativeName::new()
                        .dns(CLIENT_SUBJECT_ALT_NAME)
                        .build(&req_builder.x509v3_context(None))?,
                )?;

                req_builder.add_extensions(&extensions)?;

                req_builder.sign(&key_pair, MessageDigest::sha256())?;
                let req = req_builder.build();

                (req, key_pair)
            };

            let (client_crt_signed, client_serial_number) = {
                let mut cert_builder = X509::builder()?;
                cert_builder.set_version(2)?;

                let (serial_number, serial_number_bytes) =
                    generate_serial_number()?;
                cert_builder.set_serial_number(&serial_number)?;

                cert_builder.set_subject_name(client_csr.subject_name())?;
                cert_builder.set_issuer_name(ca_cert.subject_name())?;
                cert_builder.set_pubkey(&client_key_pair)?;

                let not_before = Asn1Time::days_from_now(NOT_BEFORE_DAYS)?;
                cert_builder.set_not_before(&not_before)?;

                let not_after =
                    Asn1Time::days_from_now(NOT_BEFORE_DAYS + DAYS_VALID)?;
                cert_builder.set_not_after(&not_after)?;

                cert_builder
                    .append_extension(BasicConstraints::new().build()?)?;

                cert_builder.append_extension(
                    KeyUsage::new()
                        .digital_signature()
                        .non_repudiation()
                        .key_encipherment()
                        .data_encipherment()
                        .build()?,
                )?;

                let auth_key_identifier = AuthorityKeyIdentifier::new()
                    .keyid(false)
                    .issuer(false)
                    .build(
                        &cert_builder.x509v3_context(Some(&ca_cert), None),
                    )?;
                cert_builder.append_extension(auth_key_identifier)?;

                cert_builder.sign(&ca_key_pair, MessageDigest::sha256())?;
                let cert = cert_builder.build();
                let cert = cert.to_pem()?;

                (cert, serial_number_bytes)
            };

            // sanity check
            assert_ne!(ca_serial_number, client_serial_number);

            return Ok(Self {
                ca_crt: ca_crt_pem,
                ca_serial_number,
                client_crt_signed,
                client_crt_signed_serial_number: client_serial_number,
            });

            fn build_subject_name(cn: &str) -> Result<X509Name> {
                let mut x509_name = X509NameBuilder::new()?;
                x509_name.append_entry_by_text("C", DISTINGUISHED_NAME_C)?;
                x509_name.append_entry_by_text("ST", DISTINGUISHED_NAME_ST)?;
                x509_name.append_entry_by_text("L", DISTINGUISHED_NAME_L)?;
                x509_name.append_entry_by_text("O", DISTINGUISHED_NAME_O)?;
                x509_name.append_entry_by_text("OU", DISTINGUISHED_NAME_OU)?;
                x509_name.append_entry_by_text("CN", cn)?;
                Ok(x509_name.build())
            }

            fn generate_serial_number() -> Result<(Asn1Integer, Vec<u8>)> {
                let mut serial = BigNum::new()?;
                serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
                let serial_number_bytes = serial.to_vec();
                let serial = serial.to_asn1_integer()?;
                Ok((serial, serial_number_bytes))
            }
        }
    }

    fn expected_issuer() -> DistinguishedName {
        DistinguishedName {
            countries: vec![DISTINGUISHED_NAME_C.into()],
            states_or_provinces: vec![DISTINGUISHED_NAME_ST.into()],
            localities: vec![DISTINGUISHED_NAME_L.into()],
            organizations: vec![DISTINGUISHED_NAME_O.into()],
            organization_units: vec![DISTINGUISHED_NAME_OU.into()],
            common_names: vec![CA_DISTINGUISHED_NAME_CN.into()],
        }
    }

    #[test]
    fn test_ca_crt() {
        let TestCerts { ca_crt, ca_serial_number, .. } =
            TestCerts::new().expect("failed to generate test certs");

        let cert =
            X509Certificate::from_pem(&ca_crt).expect("failed to parse ca_crt");

        assert_eq!(cert.version, Version::V3);
        assert_eq!(cert.serial_number, ca_serial_number);

        assert_eq!(
            cert.signature_algorithm,
            SignatureAlgorithm::Sha256WithRSAEncryption
        );

        let approximate_not_before = OffsetDateTime::now_utc()
            .add(Duration::days(NOT_BEFORE_DAYS as i64));
        let diff = approximate_not_before.sub(cert.validity_not_before).abs();
        assert!(diff < Duration::seconds(5));

        let expected_not_after =
            cert.validity_not_before.add(Duration::days(DAYS_VALID as i64));
        assert_eq!(cert.validity_not_after, expected_not_after);

        let expected = expected_issuer();
        assert_eq!(cert.issuer, expected);
        // same as issuer in ca.crt
        assert_eq!(cert.subject, expected);

        assert_eq!(cert.subject_alternative_names.len(), 1);

        // Seems like "DNS:" is converted into a byte tag
        // assert!(matches!(
        //     cert.subject_alternative_names.first(),
        //     Some(expected) if expected == "DNS:unsafe.aurae.io",
        // ));
    }

    #[test]
    fn test_client_crt() {
        let TestCerts {
            client_crt_signed,
            client_crt_signed_serial_number,
            ..
        } = TestCerts::new().expect("failed to generate test certs");

        let cert = X509Certificate::from_pem(&client_crt_signed)
            .expect("failed to parse signed client cert");

        assert_eq!(cert.version, Version::V3);
        assert_eq!(cert.serial_number, client_crt_signed_serial_number);

        assert_eq!(
            cert.signature_algorithm,
            SignatureAlgorithm::Sha256WithRSAEncryption
        );

        let approximate_not_before = OffsetDateTime::now_utc()
            .add(Duration::days(NOT_BEFORE_DAYS as i64));
        let diff = approximate_not_before.sub(cert.validity_not_before).abs();
        assert!(diff < Duration::seconds(5));

        let expected_not_after =
            cert.validity_not_before.add(Duration::days(DAYS_VALID as i64));
        assert_eq!(cert.validity_not_after, expected_not_after);

        let mut expected = expected_issuer();
        assert_eq!(cert.issuer, expected);

        expected.common_names = vec![CLIENT_DISTINGUISHED_NAME_CN.into()];
        assert_eq!(cert.subject, expected);

        assert!(cert.subject_alternative_names.is_empty());
    }
}
