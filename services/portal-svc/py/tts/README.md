# Text-to-Speech (TTS) Service

Text-to-speech service for referendum explainer audio content. Converts multilingual explainer text to audio files (MP3/OGG) via cloud TTS or on-premise NLLB audio model.

## Overview

Module: M68 (Referendum Explainer)

Audio production service for generating multi-language referendum explainer content. WCAG AAA audio requirement mandates coverage of all 11 languages.

## FastAPI Endpoints

- POST `/tts/generate` - Generate audio from text
- GET `/tts/{audio_id}` - Retrieve generated audio
- GET `/health` - Service health check

## Pydantic Models

- `TTSRequest` - Audio request (text: str, language: str, format: str = 'mp3')
- `TTSResponse` - Audio response (audio_id: str, url: str, duration_secs: float)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `TTS_PROVIDER` - Provider selection (cloud | local)
- `TTS_API_KEY` - Cloud TTS API key (Google Cloud, AWS Polly, etc.)
- `NLLB_TTS_URL` - On-prem NLLB audio model endpoint (if local)
- `S3_BUCKET` - Audio file storage bucket
- `CDN_BASE_URL` - CDN URL for audio delivery

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
gunicorn tts.wsgi:application --workers 2
```

## Supported Languages

All 11 Kenyan/regional languages (WCAG AAA compliance):
Swahili, English, Kikuyu, Luhya, Luo, Kalenjin, Kamba, Kisii, Meru, Somali, Maasai

## Providers

- **Cloud**: Google Cloud TTS (natural, scalable)
- **Local**: On-prem NLLB audio model (private, latency-sensitive)

## Notes

- Default provider: Cloud (Google Cloud TTS)
- Large batch generation (1000+ files): ~2-3 hours
- Audio is cached by language + text hash
- CDN accelerates delivery from CDN edge nodes
- Quality tier: Premium (natural voice)
