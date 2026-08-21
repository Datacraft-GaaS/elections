use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::{
	extract::{Query, State},
	response::sse::{Event, KeepAlive, Sse},
};
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::info;

use crate::sse::ShardedBroadcast;

#[derive(Debug, Deserialize)]
pub struct SseQuery {
	pub station: Option<String>,
}

pub async fn sse_events(
	State(broadcast): State<Arc<ShardedBroadcast>>,
	Query(params): Query<SseQuery>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
	let station = params.station.unwrap_or_else(|| "default".to_string());
	info!(station, "sse.subscribe");

	let rx = broadcast.subscribe(&station);
	let station_clone = station.clone();

	let stream = BroadcastStream::new(rx)
		.filter_map(move |msg| {
			let station_ref = station_clone.clone();
			match msg {
				Ok(evt) if evt.station_code == station_ref || station_ref == "default" => {
					Some(Ok(Event::default().data(evt.payload)))
				}
				_ => None,
			}
		});

	Sse::new(stream).keep_alive(
		KeepAlive::new()
			.interval(Duration::from_secs(15))
			.text("ping"),
	)
}
