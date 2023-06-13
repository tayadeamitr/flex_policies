use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use log::info;
use serde::{Deserialize, Serialize};

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(HttpConfigHeaderRoot {
            header_content: String::new()
        })
    });
}}

struct HttpConfigHeader {
    header_content: String
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
        self.add_http_response_header("custom-header",self.header_content.as_str());
        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        info!("on_http_response_body");
        Action::Continue
    }
}

#[derive(Serialize, Deserialize)]
struct CustomHeader {
     #[serde(alias = "header-content")]
    header_content: String
}

struct HttpConfigHeaderRoot {
    header_content: String
}

impl Context for HttpConfigHeaderRoot {}

impl RootContext for HttpConfigHeaderRoot {
    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            let config:CustomHeader = serde_json::from_slice(config_bytes.as_slice()).unwrap();
            self.header_content = config.header_content;
            info!("header content is {}",self.header_content);
        }
        true
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpConfigHeader {
            header_content: self.header_content.clone(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}
