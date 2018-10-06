use futures::{Future, Stream};

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

pub enum Mode {
    Get,
    Update,
}

pub struct Verbs {
    mode: Mode,
}

pub struct Node {
    name: String,
    description: String,
    verbs: Verbs,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeKind {
    name: String,
    description: Option<String>,
}

impl NodeKind {
    pub fn new(name: impl Into<String>, description: Option<String>) -> Self {
        Self {
            name: name.into(),
            description,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Method {
    name: String,
    description: Option<String>,
    arguments: Vec<String>,
}

impl Method {
    pub fn new(
        name: impl Into<String>,
        description: Option<String>,
        arguments: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            name: name.into(),
            description,
            arguments: arguments.into_iter().collect(),
        }
    }
}

pub trait Node {
    fn node_kind(&self) -> NodeKind;

    fn getters(&self) -> Vec<Method>;
    fn do_getter(
        &self,
        name: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<serde_json::Value>;

    fn updaters(&self) -> Vec<Method>;
    fn do_updater(
        &mut self,
        name: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<serde_json::Value>;
}

use actix_web::{
    self, http,
    server::{self, HttpHandler, HttpHandlerTask},
    App, HttpRequest, HttpResponse, Json, Path,
};

pub struct NodeApp {
    node: Arc<Mutex<dyn Node + Send + Sync>>,
}
pub fn create_app(name: &str, node: Arc<Mutex<dyn Node + Send + Sync>>) -> App<NodeApp> {
    App::with_state(NodeApp { node })
        .prefix(name)
        .resource("/", |r| {
            r.method(http::Method::GET)
                .f(|r| Json(r.state().node.lock().unwrap().node_kind()))
        }).resource("", |r| {
            r.method(http::Method::GET)
                .f(|r| Json(r.state().node.lock().unwrap().node_kind()))
        }).resource("/getters", |r| {
            r.method(http::Method::GET)
                .f(|r| Json(r.state().node.lock().unwrap().getters()))
        }).resource("getters/{getter_name}", |r| {
            r.method(http::Method::GET).f(|r| -> actix_web::Result<_> {
                let getter_name: String = r.match_info().query("getter_name")?;

                Ok(Json(
                    r.state()
                        .node
                        .lock()
                        .unwrap()
                        .do_getter(&getter_name, &r.query())
                        .unwrap(),
                ))
            })
        }).resource("/updaters", |r| {
            r.method(http::Method::GET)
                .f(|r| Json(r.state().node.lock().unwrap().updaters()))
        }).resource("updaters/{updater_name}", |r| {
            r.method(http::Method::POST).f(|r| -> actix_web::Result<_> {
                let updater_name: String = r.match_info().query("updater_name")?;

                Ok(Json(
                    r.state()
                        .node
                        .lock()
                        .unwrap()
                        .do_updater(&updater_name, &r.query())
                        .unwrap(),
                ))
            })
        })
}
