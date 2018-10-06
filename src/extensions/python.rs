use cpython::{NoArgs, PyDict, PyModule, PyObject, PyType, Python};
use crate::node::*;
use std::collections::HashMap;

use std::path::PathBuf;

pub struct PythonNode {
    libfledge_nodes: PyModule,
    object: PyObject,
}

impl PythonNode {
    pub fn load(
        libfledge_src: impl Into<PathBuf>,
        class_name: impl Into<String>,
        settings: serde_json::Value,
    ) -> Self {
        let libfledge_src = libfledge_src.into();
        let class_name = class_name.into();

        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);
        locals
            .set_item(py, "sys", py.import("sys").unwrap())
            .unwrap();

        py.run(
            &format!("sys.path.insert(0, '{}')", libfledge_src.display()),
            None,
            Some(&locals),
        ).unwrap();

        let libfledge_nodes = py.import("libfledge.nodes").unwrap();

        let class: PyType = libfledge_nodes
            .get(py, &class_name)
            .unwrap()
            .extract(py)
            .unwrap();

        let object = class.call(py, NoArgs, None).unwrap();

        Self {
            libfledge_nodes,
            object,
        }
    }
}

impl Node for PythonNode {
    fn node_kind(&self) -> NodeKind {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);

        locals.set_item(py, "node", &self.object).unwrap();

        locals
            .set_item(
                py,
                "kind_name",
                self.libfledge_nodes.get(py, "kind_name").unwrap(),
            ).unwrap();

        locals
            .set_item(
                py,
                "kind_description",
                self.libfledge_nodes.get(py, "kind_description").unwrap(),
            ).unwrap();

        let name: String = py
            .eval("kind_name(node)", None, Some(&locals))
            .unwrap()
            .extract(py)
            .expect("Cannot get the name of the node");

        NodeKind::new(name, None)
    }

    fn getters(&self) -> Vec<Method> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);

        locals.set_item(py, "node", &self.object).unwrap();

        locals
            .set_item(
                py,
                "node_getters",
                self.libfledge_nodes.get(py, "node_getters").unwrap(),
            ).unwrap();

        let data: Vec<(String, Option<String>, Vec<String>)> = py
            .eval("node_getters(node)", None, Some(&locals))
            .unwrap()
            .extract(py)
            .unwrap();

        data.into_iter()
            .map(|(name, description, arguments)| Method::new(name, description, arguments))
            .collect()
    }

    fn do_getter(&self, name: &str, args: &HashMap<String, String>) -> Result<serde_json::Value> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);

        locals.set_item(py, "node", &self.object).unwrap();

        locals
            .set_item(
                py,
                "do_node_getter",
                self.libfledge_nodes.get(py, "do_node_getter").unwrap(),
            ).unwrap();

        let data: String = py
            .eval(
                &format!(
                    "do_node_getter(node, '{}', {{ {} }})",
                    name,
                    args.iter()
                        .map(|(name, value)| format!("'{}': '{}',", name, value))
                        .collect::<String>()
                ),
                None,
                Some(&locals),
            ).unwrap()
            .extract(py)
            .unwrap();

        Ok(serde_json::from_str(&data).unwrap())
    }

    fn updaters(&self) -> Vec<Method> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);

        locals.set_item(py, "node", &self.object).unwrap();

        locals
            .set_item(
                py,
                "node_updaters",
                self.libfledge_nodes.get(py, "node_updaters").unwrap(),
            ).unwrap();

        let data: Vec<(String, Option<String>, Vec<String>)> = py
            .eval("node_updaters(node)", None, Some(&locals))
            .unwrap()
            .extract(py)
            .unwrap();

        data.into_iter()
            .map(|(name, description, arguments)| Method::new(name, description, arguments))
            .collect()
    }

    fn do_updater(
        &mut self,
        name: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<serde_json::Value> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);

        locals.set_item(py, "node", &self.object).unwrap();

        locals
            .set_item(
                py,
                "do_node_updater",
                self.libfledge_nodes.get(py, "do_node_updater").unwrap(),
            ).unwrap();

        let data: String = py
            .eval(
                &format!(
                    "do_node_updater(node, '{}', {{ {} }})",
                    name,
                    arguments
                        .iter()
                        .map(|(name, value)| format!("'{}': '{}',", name, value))
                        .collect::<String>()
                ),
                None,
                Some(&locals),
            ).unwrap()
            .extract(py)
            .unwrap();

        Ok(serde_json::from_str(&data).unwrap())
    }
}
