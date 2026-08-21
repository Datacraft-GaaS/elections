from __future__ import annotations
import base64
from hashlib import sha256
from ..models import FingerprintTemplate, HealthStatus, Modality
import structlog

_log = structlog.get_logger()

try:
    import skimage.morphology as _skmorph  # type: ignore[import-untyped]
    import numpy as _np  # type: ignore[import-untyped]

    SKIMAGE_AVAILABLE = True
    SKLEARN_AVAILABLE = True
except ImportError:
    _skmorph = None  # type: ignore[assignment]
    SKIMAGE_AVAILABLE = False
    try:
        import scipy.ndimage as _ndimage  # type: ignore[import-untyped]
        import numpy as _np  # type: ignore[import-untyped]

        SKLEARN_AVAILABLE = True
    except ImportError:
        _ndimage = None  # type: ignore[assignment]
        _np = None  # type: ignore[assignment]
        SKLEARN_AVAILABLE = False

if not SKIMAGE_AVAILABLE:
    _ndimage_local = None if not SKLEARN_AVAILABLE else globals().get("_ndimage")


class PyFingerprintAdapter:
    """Fingerprint adapter using skimage or scipy for basic feature extraction."""

    def _log_extract(self, image_len: int) -> None:
        _log.debug("py_fingerprint_adapter.extract", image_len=image_len)

    def extract_fingerprint(self, image_b64: str) -> FingerprintTemplate:
        if not SKLEARN_AVAILABLE or _np is None:
            raise ImportError("skimage or scipy required; pip install scikit-image")
        self._log_extract(len(image_b64))
        raw = base64.b64decode(image_b64)
        arr = _np.frombuffer(raw, dtype=_np.uint8).astype(_np.float32)

        if SKIMAGE_AVAILABLE and _skmorph is not None:
            binary = arr > 128
            skeleton = _skmorph.skeletonize(binary.reshape(1, -1)).flatten()
            processed = skeleton.astype(_np.uint8).tobytes()
        else:
            import scipy.ndimage as _nd  # type: ignore[import-untyped]

            smoothed = _nd.uniform_filter(arr, size=3)
            processed = smoothed.astype(_np.uint8).tobytes()

        data = sha256(processed).hexdigest()
        variance = float(arr.var())
        quality = min(1.0, variance / 10000.0)
        return FingerprintTemplate(
            modality=Modality.fingerprint,
            data=data,
            quality=quality,
            vendor="py",
        )

    def health(self) -> HealthStatus:
        return HealthStatus(ok=True, vendor="py", mode="py")
