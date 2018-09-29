use crate::node::*;

use serde_json::Value;
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ValueNode {
    value: String,
}

impl ValueNode {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Node for ValueNode {
    fn node_kind(&self) -> NodeKind {
        NodeKind::new("ValueNode", None)
    }

    fn getters(&self) -> Vec<Method> {
        vec![Method::new("value", None, None)]
    }

    fn do_getter(&self, name: &str, _: &HashMap<String, String>) -> Result<serde_json::Value> {
        match name {
            "value" => Ok(serde_json::Value::String(self.value.clone())),
            _ => Ok(serde_json::Value::Null),
        }
    }

    fn updaters(&self) -> Vec<Method> {
        vec![Method::new("value", None, None)]
    }

    fn do_updater(
        &mut self,
        name: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<serde_json::Value> {
        match name {
            "value" => {
                self.value = arguments["value"].clone();
                Ok(serde_json::Value::String(self.value.clone()))
            }
            _ => Ok(serde_json::Value::Null),
        }
    }
}
