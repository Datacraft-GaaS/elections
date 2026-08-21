# Turnout Forecasting

Turnout forecasting ML service providing hourly turnout predictions by constituency. Uses historical election data, weather forecasts from KMD (Kenya Meteorological Department), and real-time BVI/queue events to produce variance alerts and public dashboard feeds.

## Overview

Module: M44 (Turnout Forecasting)

Predictive analytics for voter turnout, helping election management identify underperforming areas and optimize resource allocation during election day.

## FastAPI Endpoints

- POST `/forecasts` - Generate new forecast
- GET `/forecasts/{election_id}/latest` - Latest forecast release
- GET `/forecasts/{id}/variance` - Retrieve variance alert details
- GET `/health` - Service health check

## Pydantic Models

- `ForecastRelease` - Hourly forecast (election_id, constituency_code, predicted_turnout: float, ci_lower, ci_upper, released_at)
- `HourlyTurnoutReport` - Actual turnout data (election_id, hour, actual_turnout, stations_reporting)
- `VarianceAlert` - Anomaly alert (forecast_id, actual, expected, delta, severity)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `KAFKA_BROKERS` - Kafka broker endpoints
- `KMD_ADAPTER_URL` - Kenya Meteorological Department weather API
- `BVI_API_URL` - Biometric Verification Instance (voter verification volume)
- `QUEUE_API_URL` - Queue transparency service (polling station queue lengths)
- `MODEL_ARTIFACT_PATH` - Path to trained forecast models (S3 or local)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m turnout_forecast.worker
```

## Notes

- Forecasts are released hourly during election day (7am-6pm)
- Weather data (rainfall, temperature) impacts rural turnout predictions
- Variance > 10% delta triggers alert for ECC investigation
- Model is retrained every election cycle with latest historical data
- Predictions are probabilistic (95% confidence intervals provided)
