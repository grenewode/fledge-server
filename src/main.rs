extern crate actix_web;
extern crate cpython;
extern crate futures;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod extensions;
pub mod node;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
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

pub struct AppState {
    pub nodes: Vec<Arc<Mutex<node::Node>>>,
}

fn main() {
    use actix_web::dev::Handler;
    use actix_web::{
        self, http,
        server::{self, HttpHandler, HttpHandlerTask},
        App, HttpRequest, HttpResponse, Json,
    };
    use crate::node::Node;

    server::new(|| {
        let noaa = Arc::new(Mutex::new(extensions::python::PythonNode::load(
            "/home/rmiller/git/Fledge/libfledge",
            "NOAA",
            serde_json::Value::Null,
        )));

        node::create_app(noaa)
    }).bind("127.0.0.1:8088")
    .unwrap()
    .run();
}
