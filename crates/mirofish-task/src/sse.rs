//! SSE (Server-Sent Events) streaming for task progress

use std::convert::Infallible;
use std::time::Duration;

use axum::{
    response::sse::{Event, Sse},
};
use futures::stream::Stream;
use tokio::sync::watch;

use mirofish_core::Task;

/// Create an SSE stream for a specific task
pub fn task_sse_stream(
    receiver: watch::Receiver<Task>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut rx = receiver;
        loop {
            match tokio::time::timeout(Duration::from_secs(60), rx.changed()).await {
                Ok(Ok(_)) => {
                    // Clone the data we need before the yield
                    let task_data = {
                        let task = rx.borrow_and_update();
                        serde_json::json!({
                            "task_id": task.task_id,
                            "task_type": task.task_type,
                            "status": task.status.to_string(),
                            "progress": task.progress,
                            "message": task.message.clone(),
                            "progress_detail": task.progress_detail.clone(),
                            "result": task.result.clone(),
                            "error": task.error.clone(),
                            "created_at": task.created_at,
                            "updated_at": task.updated_at,
                        })
                    };

                    let event = Event::default()
                        .event("task_update")
                        .json_data(&task_data)
                        .unwrap_or_else(|_| Event::default().data("error"));
                    yield Ok(event);

                    // Check if task is completed or failed
                    let current_status = rx.borrow().status.clone();
                    if current_status == mirofish_core::TaskStatus::Completed
                        || current_status == mirofish_core::TaskStatus::Failed
                    {
                        yield Ok(Event::default().event("task_end").data("done"));
                        break;
                    }
                }
                Ok(Err(_)) => {
                    // Channel closed
                    break;
                }
                Err(_) => {
                    // Timeout - send keep-alive
                    yield Ok(Event::default().event("keep_alive").data("ping"));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// Generic SSE stream for any watch channel
pub fn watch_stream<T>(
    mut receiver: watch::Receiver<T>,
    event_name: &'static str,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>>
where
    T: serde::Serialize + Clone + Send + Sync + 'static,
{
    let stream = async_stream::stream! {
        loop {
            match tokio::time::timeout(Duration::from_secs(60), receiver.changed()).await {
                Ok(Ok(_)) => {
                    // Clone data before yield to avoid borrow across await
                    let data = receiver.borrow_and_update().clone();
                    if let Ok(json) = serde_json::to_value(&data) {
                        if let Ok(event) = Event::default().event(event_name).json_data(&json) {
                            yield Ok(event);
                        }
                    }
                }
                Ok(Err(_)) => break,
                Err(_) => {
                    yield Ok(Event::default().event("keep_alive").data("ping"));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}