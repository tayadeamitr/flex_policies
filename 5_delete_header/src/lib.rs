use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
// use serde::{Deserialize, Serialize};//not needed as we are not taking json input

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(HttpHeaderRoot {})
    });
}}

//root context
struct HttpHeaderRoot;

impl Context for HttpHeaderRoot {}

impl RootContext for HttpHeaderRoot {
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpRemoveHeader))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

//http context
struct HttpRemoveHeader;

impl Context for HttpRemoveHeader {}

impl HttpContext for HttpRemoveHeader {
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
        self.set_http_request_header("x-custom-header", None);
        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        info!("on_http_response_body");
        Action::Continue
    }
}
