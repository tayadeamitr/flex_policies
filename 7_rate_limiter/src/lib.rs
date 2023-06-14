use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
// use serde::{Deserialize, Serialize};
// use serde_json::{json, Value};
use std::collections::HashMap;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext>{
        Box::new(RateLimitRoot{
            rate_limit: 100,
            window_size: 60, //in seconds
            request_counts: HashMap::new()
        })
    });
}}

// Root context
struct RateLimitRoot {
    rate_limit: u32,
    window_size: u32,
    request_counts: HashMap<String, u32>,
}

impl Context for RateLimitRoot {}

impl RootContext for RateLimitRoot {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        true
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(RateLimitPolicy {
            rate_limit: self.rate_limit,
            window_size: self.window_size,
            request_counts: self.request_counts.clone(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

// Http context
struct RateLimitPolicy {
    rate_limit: u32,
    window_size: u32,
    request_counts: HashMap<String, u32>,
}

impl Context for RateLimitPolicy {}

impl HttpContext for RateLimitPolicy {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        //some client identifier
        let client_id = self
            .get_http_request_header("x-client-id")
            .unwrap_or_default();

        // check for count
        let request_count = self.get_request_count(&client_id);

        // Rejecting requests if rate limit is excided.
        if request_count > self.rate_limit {
            info!("Rate limit exceede for client: {}", client_id);
            self.send_http_response(
                429,
                vec![("Content-type", "text/plain")],
                Some(b"Rate limit exceed. try after 1 minute"),
            );
            return Action::Pause;
        }
        self.update_request_count(&client_id, request_count + 1);

        Action::Continue
    }
}

impl RateLimitPolicy {
    fn get_request_count(&self, client_id: &str) -> u32 {
        *self.request_counts.get(client_id).unwrap_or(&0)
    }
    fn update_request_count(&mut self, client_id: &str, count: u32) {
        self.request_counts.insert(client_id.to_owned(), count);
    }
}
