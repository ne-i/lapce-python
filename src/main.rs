use anyhow::{anyhow, Result};
use lapce_plugin::{
    psp_types::{
        lsp_types::{
            request::Initialize, DocumentFilter, DocumentSelector, InitializeParams,
            InitializeResult, Url,
        },
        Request,
    },
    register_plugin, LapcePlugin, PLUGIN_RPC,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default)]
struct State {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    arch: String,
    os: String,
    configuration: Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    language_id: String,
    lsp_exec: Option<String>,
    options: Option<Value>,
}

register_plugin!(State);

macro_rules! ok {
    ( $x:expr ) => {
        match ($x) {
            Ok(v) => v,
            Err(e) => return Err(anyhow!(e)),
        }
    };
}

fn initialize(params: InitializeParams) -> Result<()> {
    // PLUGIN_RPC.stderr("Initializing python-lapce");

    let document_selector: DocumentSelector = vec![DocumentFilter {
        language: Some(String::from("python")),
        pattern: Some(String::from("**.py")),
        scheme: None,
    }];

    let mut server_args = vec![];
    let mut lsp_server_path = Url::parse("urn:pylsp")?;

    if let Some(options) = params.initialization_options.as_ref() {
        if let Some(pylsp) = options.get("volt") {
            if let Some(args) = pylsp.get("serverArgs") {
                if let Some(args) = args.as_array() {
                    for arg in args {
                        if let Some(arg) = arg.as_str() {
                            server_args.push(arg.to_string());
                        }
                    }
                }
            }
            if let Some(server_path) = pylsp.get("serverPath") {
                if let Some(server_path) = server_path.as_str() {
                    if !server_path.is_empty() {
                        lsp_server_path = ok!(Url::parse(&format!("urn:{}", server_path)));
                    }
                }
            }
        }
    }

    // PLUGIN_RPC.stderr(&format!("path: {}", lsp_server_path));
    // PLUGIN_RPC.stderr(&format!("args: {:?}", server_args));

    PLUGIN_RPC.start_lsp(
        lsp_server_path,
        server_args,
        document_selector,
        params.initialization_options,
    );

    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                let _ = initialize(params);
                // we need success response because of this:
                // https://github.com/lapce/lapce/pull/2087
                // see https://github.com/hbina/lapce-rust/blob/hbina-refactor-to-latest-lib/src/main.rs
                PLUGIN_RPC.host_success(_id, InitializeResult::default())
            }
            _ => {}
        }
    }
}
