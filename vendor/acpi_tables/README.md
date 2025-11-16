# acpi_tables

## Design

This crate provides the ability to generate static tables (e.g. FADT/FACP,
MCFG, etc) as well as generate AML for filling a DSDT table.

## Usage

* `aml` provides the ability to generate AML code, see the chapter titled "ACPI
  Machine Language (AML) Specification" in the ACPI Specification.
* `bert` contains routines for creating a `BERT` table
* `cedt` contains routines for creating a `CEDT` table (see the CXL specification
  for more details)
* `facs` contains routines for creating a `FACS` table
* `fadt` contains routines for creating a `FADT` table (also known as FACP)
* `hmat` contains routines for creating a `HMAT` table
* `hest` contains routines for creating a `HEST` table, except for the
  parts that are specified by UEFI
* `madt` contains routines for creating an `MADT` table (also known as APIC)
* `mcfg` contains routines for creating an `MCFG` table
* `pptt` contains routines for creating a `PPTT` table
* `rhct` contains routines for creating a `RHCT` table
* `rimt` contains routines for creating a `RIMT` table
* `rqsc` contains routines for creating a `RQSC` table
* `rsdp` contains a helper for creating a `RSDP` table
* `sdt` provides the ability to build user defined tables including header and
  checksum validation
* `slit` contains routines for creating a `SLIT` table
* `spcr` contains routines for creating a `SPCR` table (not included in ACPI specification, see [0] for details)
* `srat` contains routines for creating a `SRAT` table
* `tpm2` contains routines for creating both `TCPA` (TPM 1.2) and `TPM2` tables
* `viot` contains routines for creating a `VIOT` table
* `xsdt` contains routines for creating an `XSDT` table

## Examples

The crate is currently used by the Cloud Hypervisor project so detailed
examples of populating different ACPI table types can be found there.


## Licence

This crate is licensed under the Apache 2.0 licence. The full text can be found
in the LICENSE-APACHE file.

## Links

[0]: https://learn.microsoft.com/en-us/windows-hardware/drivers/serports/serial-port-console-redirection-table
