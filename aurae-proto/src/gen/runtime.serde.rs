// @generated
impl serde::Serialize for AllocateCellRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.cell.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.AllocateCellRequest", len)?;
        if let Some(v) = self.cell.as_ref() {
            struct_ser.serialize_field("cell", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AllocateCellRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cell",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Cell,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "cell" => Ok(GeneratedField::Cell),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AllocateCellRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.AllocateCellRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AllocateCellRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cell__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Cell => {
                            if cell__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cell"));
                            }
                            cell__ = map.next_value()?;
                        }
                    }
                }
                Ok(AllocateCellRequest {
                    cell: cell__,
                })
            }
        }
        deserializer.deserialize_struct("runtime.AllocateCellRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AllocateCellResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.cell_name.is_empty() {
            len += 1;
        }
        if self.cgroup_v2 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.AllocateCellResponse", len)?;
        if !self.cell_name.is_empty() {
            struct_ser.serialize_field("cellName", &self.cell_name)?;
        }
        if self.cgroup_v2 {
            struct_ser.serialize_field("cgroupV2", &self.cgroup_v2)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AllocateCellResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cell_name",
            "cellName",
            "cgroup_v2",
            "cgroupV2",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CellName,
            CgroupV2,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "cellName" | "cell_name" => Ok(GeneratedField::CellName),
                            "cgroupV2" | "cgroup_v2" => Ok(GeneratedField::CgroupV2),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AllocateCellResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.AllocateCellResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AllocateCellResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cell_name__ = None;
                let mut cgroup_v2__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CellName => {
                            if cell_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellName"));
                            }
                            cell_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::CgroupV2 => {
                            if cgroup_v2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cgroupV2"));
                            }
                            cgroup_v2__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AllocateCellResponse {
                    cell_name: cell_name__.unwrap_or_default(),
                    cgroup_v2: cgroup_v2__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("runtime.AllocateCellResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Cell {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.cpu_cpus.is_empty() {
            len += 1;
        }
        if self.cpu_shares != 0 {
            len += 1;
        }
        if !self.cpu_mems.is_empty() {
            len += 1;
        }
        if self.cpu_quota != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.Cell", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.cpu_cpus.is_empty() {
            struct_ser.serialize_field("cpuCpus", &self.cpu_cpus)?;
        }
        if self.cpu_shares != 0 {
            struct_ser.serialize_field("cpuShares", ToString::to_string(&self.cpu_shares).as_str())?;
        }
        if !self.cpu_mems.is_empty() {
            struct_ser.serialize_field("cpuMems", &self.cpu_mems)?;
        }
        if self.cpu_quota != 0 {
            struct_ser.serialize_field("cpuQuota", ToString::to_string(&self.cpu_quota).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Cell {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "cpu_cpus",
            "cpuCpus",
            "cpu_shares",
            "cpuShares",
            "cpu_mems",
            "cpuMems",
            "cpu_quota",
            "cpuQuota",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            CpuCpus,
            CpuShares,
            CpuMems,
            CpuQuota,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "cpuCpus" | "cpu_cpus" => Ok(GeneratedField::CpuCpus),
                            "cpuShares" | "cpu_shares" => Ok(GeneratedField::CpuShares),
                            "cpuMems" | "cpu_mems" => Ok(GeneratedField::CpuMems),
                            "cpuQuota" | "cpu_quota" => Ok(GeneratedField::CpuQuota),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Cell;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.Cell")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Cell, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut cpu_cpus__ = None;
                let mut cpu_shares__ = None;
                let mut cpu_mems__ = None;
                let mut cpu_quota__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::CpuCpus => {
                            if cpu_cpus__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cpuCpus"));
                            }
                            cpu_cpus__ = Some(map.next_value()?);
                        }
                        GeneratedField::CpuShares => {
                            if cpu_shares__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cpuShares"));
                            }
                            cpu_shares__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CpuMems => {
                            if cpu_mems__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cpuMems"));
                            }
                            cpu_mems__ = Some(map.next_value()?);
                        }
                        GeneratedField::CpuQuota => {
                            if cpu_quota__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cpuQuota"));
                            }
                            cpu_quota__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Cell {
                    name: name__.unwrap_or_default(),
                    cpu_cpus: cpu_cpus__.unwrap_or_default(),
                    cpu_shares: cpu_shares__.unwrap_or_default(),
                    cpu_mems: cpu_mems__.unwrap_or_default(),
                    cpu_quota: cpu_quota__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("runtime.Cell", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Executable {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.command.is_empty() {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.Executable", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.command.is_empty() {
            struct_ser.serialize_field("command", &self.command)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Executable {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "command",
            "description",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Command,
            Description,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "command" => Ok(GeneratedField::Command),
                            "description" => Ok(GeneratedField::Description),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Executable;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.Executable")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Executable, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut command__ = None;
                let mut description__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Command => {
                            if command__.is_some() {
                                return Err(serde::de::Error::duplicate_field("command"));
                            }
                            command__ = Some(map.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Executable {
                    name: name__.unwrap_or_default(),
                    command: command__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("runtime.Executable", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FreeCellRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.cell_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.FreeCellRequest", len)?;
        if !self.cell_name.is_empty() {
            struct_ser.serialize_field("cellName", &self.cell_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FreeCellRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cell_name",
            "cellName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CellName,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "cellName" | "cell_name" => Ok(GeneratedField::CellName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FreeCellRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.FreeCellRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FreeCellRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cell_name__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CellName => {
                            if cell_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellName"));
                            }
                            cell_name__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(FreeCellRequest {
                    cell_name: cell_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("runtime.FreeCellRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FreeCellResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("runtime.FreeCellResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FreeCellResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FreeCellResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.FreeCellResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FreeCellResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(FreeCellResponse {
                })
            }
        }
        deserializer.deserialize_struct("runtime.FreeCellResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StartCellRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.cell_name.is_empty() {
            len += 1;
        }
        if !self.executables.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.StartCellRequest", len)?;
        if !self.cell_name.is_empty() {
            struct_ser.serialize_field("cellName", &self.cell_name)?;
        }
        if !self.executables.is_empty() {
            struct_ser.serialize_field("executables", &self.executables)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StartCellRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cell_name",
            "cellName",
            "executables",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CellName,
            Executables,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "cellName" | "cell_name" => Ok(GeneratedField::CellName),
                            "executables" => Ok(GeneratedField::Executables),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StartCellRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.StartCellRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StartCellRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cell_name__ = None;
                let mut executables__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CellName => {
                            if cell_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellName"));
                            }
                            cell_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Executables => {
                            if executables__.is_some() {
                                return Err(serde::de::Error::duplicate_field("executables"));
                            }
                            executables__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(StartCellRequest {
                    cell_name: cell_name__.unwrap_or_default(),
                    executables: executables__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("runtime.StartCellRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StartCellResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("runtime.StartCellResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StartCellResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StartCellResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.StartCellResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StartCellResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(StartCellResponse {
                })
            }
        }
        deserializer.deserialize_struct("runtime.StartCellResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StopCellRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.cell_name.is_empty() {
            len += 1;
        }
        if !self.executable_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.StopCellRequest", len)?;
        if !self.cell_name.is_empty() {
            struct_ser.serialize_field("cellName", &self.cell_name)?;
        }
        if !self.executable_name.is_empty() {
            struct_ser.serialize_field("executableName", &self.executable_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StopCellRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cell_name",
            "cellName",
            "executable_name",
            "executableName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CellName,
            ExecutableName,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "cellName" | "cell_name" => Ok(GeneratedField::CellName),
                            "executableName" | "executable_name" => Ok(GeneratedField::ExecutableName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StopCellRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.StopCellRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StopCellRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cell_name__ = None;
                let mut executable_name__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CellName => {
                            if cell_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellName"));
                            }
                            cell_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::ExecutableName => {
                            if executable_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("executableName"));
                            }
                            executable_name__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(StopCellRequest {
                    cell_name: cell_name__.unwrap_or_default(),
                    executable_name: executable_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("runtime.StopCellRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StopCellResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("runtime.StopCellResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StopCellResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StopCellResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.StopCellResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StopCellResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(StopCellResponse {
                })
            }
        }
        deserializer.deserialize_struct("runtime.StopCellResponse", FIELDS, GeneratedVisitor)
    }
}
