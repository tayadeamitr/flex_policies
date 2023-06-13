use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::Deserialize;

// [1] Main:
proxy_wasm::main! {{               //entry point macro from proxy_wasm
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(CustomAuthRootContext {     // for root context
            config: CustomAuthConfig::default(),  //attribute form json
        })
    });
}}

// [2] Root Context:
struct CustomAuthRootContext {
    config: CustomAuthConfig, //struct to store config form json
}

impl Context for CustomAuthRootContext {}

impl RootContext for CustomAuthRootContext {
    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            self.config = serde_json::from_slice(config_bytes.as_slice()).unwrap();
        }

        true
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(CustomAuthHttpContext {
            config: self.config.clone(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

// [3] Struct to store values form Json:
#[derive(Default, Clone, Deserialize)]
struct CustomAuthConfig {
    #[serde(alias = "secret-value")]
    secret_value: String,
}

// [4] Http Context Code:
struct CustomAuthHttpContext {
    pub config: CustomAuthConfig,
}

impl Context for CustomAuthHttpContext {}

impl HttpContext for CustomAuthHttpContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        if let Some(value) = self.get_http_request_header("x-custom-auth") {
            if self.config.secret_value == value {
                return Action::Continue;
            }
        }

        self.send_http_response(401, Vec::new(), None);

        Action::Pause
    }
}
