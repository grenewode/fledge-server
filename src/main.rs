extern crate actix_web;
extern crate cpython;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate env_logger;
extern crate log;
extern crate serde_json;

pub mod extensions;
pub mod node;

use crate::node::Node;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn load_value_node(path: impl AsRef<Path>) -> crate::extensions::base::ValueNode {
    use crate::extensions::base::ValueNode;
    let path = path.as_ref();

    if path.exists() {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).unwrap()
    } else {
        let value = ValueNode::default();
        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &value).unwrap();

        value
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeKindSettings {
    Python {
        #[serde(default)]
        module_path: Option<PathBuf>,
        node_name: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeSettings {
    kind: NodeKindSettings,
    #[serde(default)]
    settings: serde_json::Value,
}

impl NodeSettings {
    pub fn make_node(&self, settings: &AppSettings) -> Arc<Mutex<dyn Node + Send + Sync>> {
        match self.kind {
            NodeKindSettings::Python {
                ref module_path,
                ref node_name,
            } => Arc::new(Mutex::new(extensions::python::PythonNode::load(
                module_path
                    .as_ref()
                    .unwrap_or(&settings.libfledge_path)
                    .clone(),
                node_name.clone(),
                serde_json::Value::Null,
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    libfledge_path: PathBuf,

    #[serde(default, flatten)]
    nodes: HashMap<String, NodeSettings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootResponseNode {
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootResponse {
    nodes: Vec<RootResponseNode>,
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    use actix_web::dev::Handler;
    use actix_web::{
        self, http,
        middleware::Logger,
        server::{self, HttpHandler, HttpHandlerTask},
        App, HttpRequest, HttpResponse, Json,
    };

    lazy_static! {
        static ref SETTINGS: AppSettings = {
            let config_file = File::open("config.json").unwrap();
            let config_reader = BufReader::new(config_file);

            serde_json::from_reader(config_reader).unwrap()
        };
    }

    println!("{:#?}", *SETTINGS);

    lazy_static! {
        static ref NODES: Vec<(String, Arc<Mutex<dyn Node + Send + Sync>>)> = {
            SETTINGS
                .nodes
                .iter()
                .map(|(name, node)| (name.clone(), node.make_node(&SETTINGS)))
                .collect()
        };
    }

    server::new(|| {
        NODES
            .iter()
            .map(|(name, node)| {
                node::create_app(name, node.clone())
                    .middleware(Logger::default())
                    .boxed()
            }).chain(iter::once(
                App::new()
                    .middleware(Logger::new("BASE %r"))
                    .resource("/", |r| {
                        r.get().f(|_| {
                            Json(RootResponse {
                                nodes: NODES
                                    .iter()
                                    .map(|entry| RootResponseNode {
                                        name: entry.0.clone(),
                                        url: format!("/{}", entry.0),
                                    }).collect::<Vec<_>>(),
                            })
                        })
                    }).boxed(),
            ))
    }).bind("127.0.0.1:8088")
    .unwrap()
    .run();
}
