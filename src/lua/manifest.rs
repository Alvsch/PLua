use mlua::{FromLua, prelude::*};

pub struct LuaPluginManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub on_enable: Option<LuaFunction>,
    pub on_disable: Option<LuaFunction>,
}

impl FromLua for LuaPluginManifest {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        Ok(LuaPluginManifest {
            name: table.get("name")?,
            description: table.get("description").unwrap_or_else(|_| String::new()),
            version: table.get("version").unwrap_or_else(|_| "1.0.0".to_string()),
            author: table
                .get("author")
                .unwrap_or_else(|_| "Unknown".to_string()),
            on_enable: table.get("on_enable")?,
            on_disable: table.get("on_disable")?,
        })
    }
}
