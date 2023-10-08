use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snowflake::SnowflakeIdBucket;
use ts_rs::TS;

use std::collections::HashMap;

use crate::core::module::{GuestEvent, ModuleName};
use crate::core::Snowflake;

pub type GuestId = Snowflake;
pub type ResourceMetaName = String;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct ResourceMap(pub HashMap<ModuleName, Vec<Resource>>);

#[derive(TS, Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[ts(export)]
pub enum ResourceKind {
    Menu,
    Image,
    Font,
    TileSet(TileSetResourceDef),
    Sound,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[ts(export)]
pub struct TileSetResourceDef {
    pub(crate) start_gid: String,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct Resource {
    pub(super) meta_name: ResourceMetaName,
    pub(super) kind: ResourceKind,
    pub(super) path: String,
    pub(super) cache_hash: Snowflake,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ResourceConfig {
    pub(crate) meta_name: ResourceMetaName,
    pub(crate) kind: ResourceKind,
    pub(crate) path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ResourceFile {
    pub(crate) module_name: ModuleName,
    pub(crate) resources: Vec<ResourceConfig>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum ResourceEvent {
    LoadResource(ResourceMap),
    UnLoadResource(ModuleName),
}

pub struct ResourceModule {
    pub(super) active_resources: HashMap<GuestId, HashMap<ModuleName, bool>>,
    pub(super) resources: HashMap<ModuleName, HashMap<ResourceMetaName, Resource>>,
    pub(super) resource_load_events: Vec<GuestEvent<ResourceEvent>>,
    pub(super) resource_hash_gen: SnowflakeIdBucket,
}

#[cfg(test)]
mod tests {
    use super::ResourceFile;
    use schemars::schema_for;
    use std::fs::File;
    use std::io::Write;

    #[test]
    pub fn generate_json_schemas() {
        let resource_file_schema = schema_for!(ResourceFile);
        let mut file = File::create("schemas/resources.schema.json").unwrap();
        file.write_all(
            serde_json::to_string_pretty(&resource_file_schema)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
    }
}
