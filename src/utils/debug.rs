//! Debug utilities for the search SDK

use crate::types::DebugOptions;

/// Log a message if debugging is enabled
pub fn log(options: &Option<DebugOptions>, message: &str, data: &str) {
    if let Some(debug_opts) = options {
        if debug_opts.enabled {
            eprintln!("[search-sdk] {message}: {data}");
        }
    }
}

/// Log request details if request logging is enabled
pub fn log_request(options: &Option<DebugOptions>, message: &str, data: &str) {
    if let Some(debug_opts) = options {
        if debug_opts.enabled && debug_opts.log_requests {
            eprintln!("[search-sdk] REQUEST: {message}: {data}");
        }
    }
}

/// Log response details if response logging is enabled
pub fn log_response(options: &Option<DebugOptions>, message: &str) {
    if let Some(debug_opts) = options {
        if debug_opts.enabled && debug_opts.log_responses {
            eprintln!("[search-sdk] RESPONSE: {message}");
        }
    }
}

/// Create default debug options with all logging enabled
pub fn debug_all() -> DebugOptions {
    DebugOptions {
        enabled: true,
        log_requests: true,
        log_responses: true,
    }
}

/// Create debug options with only basic logging enabled
pub fn debug_basic() -> DebugOptions {
    DebugOptions {
        enabled: true,
        log_requests: false,
        log_responses: false,
    }
}
