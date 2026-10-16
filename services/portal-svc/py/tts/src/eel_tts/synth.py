# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""TTS adapter interface: PiperAdapter (offline, MIT) and StubAdapter (silence).

EEL_TTS_ADAPTER=stub  -> StubAdapter (default in test/CI environments)
EEL_TTS_ADAPTER=piper -> PiperAdapter (requires piper-tts installed)
"""

from __future__ import annotations

import os
from abc import ABC, abstractmethod
from pathlib import Path


class TTSAdapter(ABC):
    @abstractmethod
    async def synthesise(self, text: str, lang: str) -> bytes:
        """Return raw MP3 bytes for the given text and language code."""
        ...


class StubAdapter(TTSAdapter):
    """Returns a minimal valid MP3 header (1-second silence, ~1 KB).

    The bytes form a valid ID3v2 tag + one silent MPEG frame so consumers
    can parse the file without errors.
    """

    async def synthesise(self, text: str, lang: str) -> bytes:
        # ID3v2.3 header: 10 bytes, size 0
        id3_header = b"ID3\x03\x00\x00\x00\x00\x00\x00"
        # Minimal MPEG 1 Layer 3 frame header: 128 kbps, 44100 Hz, stereo
        # 0xFFFB = sync + MPEG1 + Layer3 + 128kbps; 0x90 = 44100 Hz + stereo
        mpeg_frame = b"\xff\xfb\x90\x00" + b"\x00" * 413  # 417-byte frame
        return id3_header + mpeg_frame


class PiperAdapter(TTSAdapter):
    """Piper TTS adapter (offline, MIT-licensed).

    Requires piper-tts to be installed: `pip install piper-tts`
    and appropriate voice model files present at EEL_PIPER_MODEL_DIR.
    """

    def __init__(self) -> None:
        try:
            from piper import PiperVoice  # type: ignore[import]

            self._voice_cls = PiperVoice
        except ImportError as exc:
            raise RuntimeError(
                "piper-tts not installed. Set EEL_TTS_ADAPTER=stub for testing "
                "or install piper-tts for production."
            ) from exc
        self._model_dir = Path(
            os.environ.get("EEL_PIPER_MODEL_DIR", "/opt/piper-models")
        )

    async def synthesise(self, text: str, lang: str) -> bytes:
        import io
        import wave

        model_path = self._model_dir / f"{lang}.onnx"
        if not model_path.exists():
            raise FileNotFoundError(f"Piper model not found: {model_path}")
        voice = self._voice_cls.load(str(model_path))
        buf = io.BytesIO()
        with wave.open(buf, "wb") as wav_file:
            wav_file.setnchannels(1)
            wav_file.setsampwidth(2)
            wav_file.setframerate(22050)
            voice.synthesize(text, wav_file)
        return buf.getvalue()


def get_adapter() -> TTSAdapter:
    """Factory: select adapter via EEL_TTS_ADAPTER env var."""
    adapter_name = os.environ.get("EEL_TTS_ADAPTER", "stub").lower()
    if adapter_name == "stub":
        return StubAdapter()
    if adapter_name == "piper":
        return PiperAdapter()
    raise ValueError(
        f"Unknown EEL_TTS_ADAPTER: {adapter_name!r}. Use 'stub' or 'piper'."
    )
