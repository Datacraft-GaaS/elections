# CV Headcount

Computer vision headcount service for estimating queue lengths at polling stations. Processes CCTV/drone frame grabs to estimate queue sizes and publishes `QueueSnapshot` events to Kafka for real-time dashboard feeds. Requires GPU for inference.

## Overview

Module: M19 (Queue Transparency)

Computer vision service for estimating voter queue lengths at polling stations. Enables real-time queue monitoring and disenfranchisement risk alerts when queues become excessive.

## FastAPI Endpoints

- POST `/cv/ingest` - Ingest frame grab for analysis
- GET `/cv/snapshot/{station_id}/latest` - Retrieve latest queue snapshot
- GET `/health` - Service health check

## Pydantic Models

- `FrameIngest` - Video frame input (station_id: str, timestamp: str, frame_b64: str)
- `QueueSnapshot` - Estimated queue (station_id, timestamp, estimated_count: int, confidence: float, disenfranchisement_flag: bool)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `KAFKA_BROKERS` - Kafka broker endpoints
- `GPU_DEVICE_ID` - CUDA device ID (default: 0)
- `MODEL_ARTIFACT_PATH` - Path to trained CV model (S3 or local)
- `CONFIDENCE_THRESHOLD` - Confidence threshold for publishing (default: 0.75)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m cv_headcount.worker
```

## GPU Requirements

- CUDA 12+ required
- Minimum 8GB VRAM (RTX 4070 or better)
- Inference time: ~200-500ms per frame

## Notes

- Input frames are CCTV (720p-1080p) or drone footage (4K)
- Confidence > 0.85 indicates reliable estimates
- Disenfranchisement flag set when queue > 100 people or wait time > 4 hours projected
- Frames are deleted after inference (no persistent video storage)
- Model is retrained weekly with new labeled frames
