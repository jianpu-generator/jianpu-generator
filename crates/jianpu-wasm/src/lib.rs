use jianpu_generator::{error_reporter, render_svgs_from_source};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct RenderOk {
    svgs: Vec<String>,
}

#[derive(Serialize)]
struct SpanOut {
    start: usize,
    end: usize,
}

#[derive(Serialize)]
struct RenderErr {
    message: String,
    span: SpanOut,
    #[serde(skip_serializing_if = "Option::is_none")]
    report: Option<String>,
}

/// Parse and render `.jianpu` source into SVG page strings.
///
/// On success returns `{ "svgs": ["<svg>...</svg>", ...] }`.
/// On error returns `{ "message": "...", "span": { "start", "end" }, "report": "..." }`.
#[wasm_bindgen]
pub fn render(source: &str) -> Result<JsValue, JsValue> {
    match render_svgs_from_source(source, "input.jianpu") {
        Ok(svgs) => Ok(serde_wasm_bindgen::to_value(&RenderOk { svgs }).map_err(js_error)?),
        Err(e) => {
            let report = error_reporter::render_with_source(source, &e);
            Err(serde_wasm_bindgen::to_value(&RenderErr {
                message: e.message.clone(),
                span: SpanOut {
                    start: e.span.start,
                    end: e.span.end,
                },
                report: Some(report),
            })
            .map_err(js_error)?)
        }
    }
}

fn js_error(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}
