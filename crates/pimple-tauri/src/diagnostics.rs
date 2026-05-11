//! Capture WARN-level tracing events whose target starts with `pimple_core`
//! and forward them to the frontend as `diagnostic` Tauri events.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use tracing::Subscriber;
use tracing::field::Visit;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

#[derive(Debug, Serialize, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub level: String,
}

pub struct ForwardingLayer<R: Runtime> {
    pub handle: AppHandle<R>,
}

impl<S, R> Layer<S> for ForwardingLayer<R>
where
    S: Subscriber,
    R: Runtime,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let target = event.metadata().target();
        if !target.starts_with("pimple_core") {
            return;
        }
        let level = event.metadata().level();
        if *level > tracing::Level::WARN {
            return;
        }
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let payload = Diagnostic {
            message: visitor.0,
            level: level.to_string(),
        };
        let _ = self.handle.emit("diagnostic", payload);
    }
}

#[derive(Default)]
struct MessageVisitor(String);

impl Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            value.clone_into(&mut self.0);
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}
