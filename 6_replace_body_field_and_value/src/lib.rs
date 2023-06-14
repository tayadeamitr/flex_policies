use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Entrypoint
proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(HttpConfigHeaderRoot {
            old_field_name: String::new(),
            new_field_name: String::new(),
            new_field_value: String::new(),

        })
    });
}}
// Alias
#[derive(Serialize, Deserialize)]
struct PolicyConfig {
    #[serde(alias = "old-field-name")]
    old_field_name: String,
    #[serde(alias = "new-field-name")]
    new_field_name: String,
    #[serde(alias = "new-field-value")]
    new_field_value: String,
}
// Root context
struct HttpConfigHeaderRoot {
    old_field_name: String,
    new_field_name: String,
    new_field_value: String,
}
impl Context for HttpConfigHeaderRoot {}

impl RootContext for HttpConfigHeaderRoot {
    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            let config: PolicyConfig = serde_json::from_slice(config_bytes.as_slice()).unwrap();
            self.old_field_name = config.old_field_name;
            self.new_field_name = config.new_field_name;
            self.new_field_value = config.new_field_value;

            info!(
                "Old field name is {}, which will be replaced with {} and value: {}",
                self.old_field_name, self.new_field_name, self.new_field_value
            );
        }
        true
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpConfigHeader {
            old_field_name: self.old_field_name.clone(),
            new_field_name: self.new_field_name.clone(),
            new_field_value: self.new_field_value.clone(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}
//HTTP context
struct HttpConfigHeader {
    old_field_name: String,
    new_field_name: String,
    new_field_value: String,
}

impl Context for HttpConfigHeader {}

impl HttpContext for HttpConfigHeader {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        info!("on_http_request_headers");
        Action::Continue
    }

    fn on_http_request_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        info!("on_http_request_body");
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        info!("on_http_response_headers");
        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        info!("on_http_response_body");
        if !_end_of_stream {
            info!("on_http_response_body wait end of stream");
            return Action::Pause;
        }

        // Replace the attribute and its value.
        if let Some(body_bytes) = self.get_http_response_body(0, _body_size) {
            info!("on_http_response_body wait read body");
            let body_str = String::from_utf8(body_bytes).unwrap();
            let body_str_new = transform(
                body_str,
                (
                    self.old_field_name.clone(),
                    self.new_field_name.clone(),
                    self.new_field_value.clone(),
                ),
            );
            self.set_http_response_body(0, _body_size, body_str_new.as_bytes());
        }
        Action::Continue
    }
}

fn transform(input: String, (old_field, new_field, new_value): (String, String, String)) -> String {
    info!("transform function");
    let mut v: Value = serde_json::from_str(&input).unwrap();
    if let Some(body_v) = v.as_object_mut() {
        if let Some(_) = body_v.remove(&old_field) {
            body_v.insert(new_field, json!(new_value));
        }
    }
    v.to_string()
}
