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
        let len = 0;
        let struct_ser = serializer.serialize_struct("runtime.AllocateCellResponse", len)?;
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
            type Value = AllocateCellResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.AllocateCellResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AllocateCellResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(AllocateCellResponse {
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
        if !self.cpus.is_empty() {
            len += 1;
        }
        if !self.mems.is_empty() {
            len += 1;
        }
        if self.shares != 0 {
            len += 1;
        }
        if self.quota != 0 {
            len += 1;
        }
        if self.ns_share_mount {
            len += 1;
        }
        if self.ns_share_uts {
            len += 1;
        }
        if self.ns_share_ipc {
            len += 1;
        }
        if self.ns_share_pid {
            len += 1;
        }
        if self.ns_share_net {
            len += 1;
        }
        if self.ns_share_cgroup {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.Cell", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.cpus.is_empty() {
            struct_ser.serialize_field("cpus", &self.cpus)?;
        }
        if !self.mems.is_empty() {
            struct_ser.serialize_field("mems", &self.mems)?;
        }
        if self.shares != 0 {
            struct_ser.serialize_field("shares", ToString::to_string(&self.shares).as_str())?;
        }
        if self.quota != 0 {
            struct_ser.serialize_field("quota", ToString::to_string(&self.quota).as_str())?;
        }
        if self.ns_share_mount {
            struct_ser.serialize_field("nsShareMount", &self.ns_share_mount)?;
        }
        if self.ns_share_uts {
            struct_ser.serialize_field("nsShareUts", &self.ns_share_uts)?;
        }
        if self.ns_share_ipc {
            struct_ser.serialize_field("nsShareIpc", &self.ns_share_ipc)?;
        }
        if self.ns_share_pid {
            struct_ser.serialize_field("nsSharePid", &self.ns_share_pid)?;
        }
        if self.ns_share_net {
            struct_ser.serialize_field("nsShareNet", &self.ns_share_net)?;
        }
        if self.ns_share_cgroup {
            struct_ser.serialize_field("nsShareCgroup", &self.ns_share_cgroup)?;
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
            "cpus",
            "mems",
            "shares",
            "quota",
            "ns_share_mount",
            "nsShareMount",
            "ns_share_uts",
            "nsShareUts",
            "ns_share_ipc",
            "nsShareIpc",
            "ns_share_pid",
            "nsSharePid",
            "ns_share_net",
            "nsShareNet",
            "ns_share_cgroup",
            "nsShareCgroup",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Cpus,
            Mems,
            Shares,
            Quota,
            NsShareMount,
            NsShareUts,
            NsShareIpc,
            NsSharePid,
            NsShareNet,
            NsShareCgroup,
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
                            "cpus" => Ok(GeneratedField::Cpus),
                            "mems" => Ok(GeneratedField::Mems),
                            "shares" => Ok(GeneratedField::Shares),
                            "quota" => Ok(GeneratedField::Quota),
                            "nsShareMount" | "ns_share_mount" => Ok(GeneratedField::NsShareMount),
                            "nsShareUts" | "ns_share_uts" => Ok(GeneratedField::NsShareUts),
                            "nsShareIpc" | "ns_share_ipc" => Ok(GeneratedField::NsShareIpc),
                            "nsSharePid" | "ns_share_pid" => Ok(GeneratedField::NsSharePid),
                            "nsShareNet" | "ns_share_net" => Ok(GeneratedField::NsShareNet),
                            "nsShareCgroup" | "ns_share_cgroup" => Ok(GeneratedField::NsShareCgroup),
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
                let mut cpus__ = None;
                let mut mems__ = None;
                let mut shares__ = None;
                let mut quota__ = None;
                let mut ns_share_mount__ = None;
                let mut ns_share_uts__ = None;
                let mut ns_share_ipc__ = None;
                let mut ns_share_pid__ = None;
                let mut ns_share_net__ = None;
                let mut ns_share_cgroup__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Cpus => {
                            if cpus__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cpus"));
                            }
                            cpus__ = Some(map.next_value()?);
                        }
                        GeneratedField::Mems => {
                            if mems__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mems"));
                            }
                            mems__ = Some(map.next_value()?);
                        }
                        GeneratedField::Shares => {
                            if shares__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shares"));
                            }
                            shares__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Quota => {
                            if quota__.is_some() {
                                return Err(serde::de::Error::duplicate_field("quota"));
                            }
                            quota__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::NsShareMount => {
                            if ns_share_mount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nsShareMount"));
                            }
                            ns_share_mount__ = Some(map.next_value()?);
                        }
                        GeneratedField::NsShareUts => {
                            if ns_share_uts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nsShareUts"));
                            }
                            ns_share_uts__ = Some(map.next_value()?);
                        }
                        GeneratedField::NsShareIpc => {
                            if ns_share_ipc__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nsShareIpc"));
                            }
                            ns_share_ipc__ = Some(map.next_value()?);
                        }
                        GeneratedField::NsSharePid => {
                            if ns_share_pid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nsSharePid"));
                            }
                            ns_share_pid__ = Some(map.next_value()?);
                        }
                        GeneratedField::NsShareNet => {
                            if ns_share_net__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nsShareNet"));
                            }
                            ns_share_net__ = Some(map.next_value()?);
                        }
                        GeneratedField::NsShareCgroup => {
                            if ns_share_cgroup__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nsShareCgroup"));
                            }
                            ns_share_cgroup__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Cell {
                    name: name__.unwrap_or_default(),
                    cpus: cpus__.unwrap_or_default(),
                    mems: mems__.unwrap_or_default(),
                    shares: shares__.unwrap_or_default(),
                    quota: quota__.unwrap_or_default(),
                    ns_share_mount: ns_share_mount__.unwrap_or_default(),
                    ns_share_uts: ns_share_uts__.unwrap_or_default(),
                    ns_share_ipc: ns_share_ipc__.unwrap_or_default(),
                    ns_share_pid: ns_share_pid__.unwrap_or_default(),
                    ns_share_net: ns_share_net__.unwrap_or_default(),
                    ns_share_cgroup: ns_share_cgroup__.unwrap_or_default(),
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
        if !self.cell_name.is_empty() {
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
        if !self.cell_name.is_empty() {
            struct_ser.serialize_field("cellName", &self.cell_name)?;
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
            "cell_name",
            "cellName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Command,
            Description,
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
                            "name" => Ok(GeneratedField::Name),
                            "command" => Ok(GeneratedField::Command),
                            "description" => Ok(GeneratedField::Description),
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
                let mut cell_name__ = None;
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
                        GeneratedField::CellName => {
                            if cell_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellName"));
                            }
                            cell_name__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Executable {
                    name: name__.unwrap_or_default(),
                    command: command__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    cell_name: cell_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("runtime.Executable", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExecutableReference {
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
        let mut struct_ser = serializer.serialize_struct("runtime.ExecutableReference", len)?;
        if !self.cell_name.is_empty() {
            struct_ser.serialize_field("cellName", &self.cell_name)?;
        }
        if !self.executable_name.is_empty() {
            struct_ser.serialize_field("executableName", &self.executable_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExecutableReference {
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
            type Value = ExecutableReference;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.ExecutableReference")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ExecutableReference, V::Error>
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
                Ok(ExecutableReference {
                    cell_name: cell_name__.unwrap_or_default(),
                    executable_name: executable_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("runtime.ExecutableReference", FIELDS, GeneratedVisitor)
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
        if self.cell.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.FreeCellRequest", len)?;
        if let Some(v) = self.cell.as_ref() {
            struct_ser.serialize_field("cell", v)?;
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
            type Value = FreeCellRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct runtime.FreeCellRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FreeCellRequest, V::Error>
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
                Ok(FreeCellRequest {
                    cell: cell__,
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
        if self.executable.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.StartCellRequest", len)?;
        if let Some(v) = self.executable.as_ref() {
            struct_ser.serialize_field("executable", v)?;
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
            "executable",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Executable,
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
                            "executable" => Ok(GeneratedField::Executable),
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
                let mut executable__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Executable => {
                            if executable__.is_some() {
                                return Err(serde::de::Error::duplicate_field("executable"));
                            }
                            executable__ = map.next_value()?;
                        }
                    }
                }
                Ok(StartCellRequest {
                    executable: executable__,
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
        if self.executable_reference.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("runtime.StopCellRequest", len)?;
        if let Some(v) = self.executable_reference.as_ref() {
            struct_ser.serialize_field("executableReference", v)?;
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
            "executable_reference",
            "executableReference",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExecutableReference,
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
                            "executableReference" | "executable_reference" => Ok(GeneratedField::ExecutableReference),
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
                let mut executable_reference__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ExecutableReference => {
                            if executable_reference__.is_some() {
                                return Err(serde::de::Error::duplicate_field("executableReference"));
                            }
                            executable_reference__ = map.next_value()?;
                        }
                    }
                }
                Ok(StopCellRequest {
                    executable_reference: executable_reference__,
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
