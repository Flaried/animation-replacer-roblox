use crate::ScriptType;

/// Used to Determine what script it is.
impl ScriptType {
    pub fn from_class_name(class_name: &str) -> Self {
        match class_name {
            "Script" => ScriptType::Script,
            "LocalScript" => ScriptType::LocalScript,
            "ModuleScript" => ScriptType::ModuleScript,
            other => ScriptType::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ScriptType::Script => "Script",
            ScriptType::LocalScript => "LocalScript",
            ScriptType::ModuleScript => "ModuleScript",
            ScriptType::Unknown(s) => s.as_str(),
        }
    }
}
