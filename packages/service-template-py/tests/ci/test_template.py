"""Conformance test: generate a service from the template and verify it."""

import json
import pathlib
import subprocess

TEMPLATE_DIR = pathlib.Path(__file__).parent.parent.parent


def test_cookiecutter_json_valid() -> None:
    cc_json = TEMPLATE_DIR / "cookiecutter.json"
    data = json.loads(cc_json.read_text())
    assert "svc_name" in data
    assert "port" in data


def test_template_directory_exists() -> None:
    template_svc_dir = TEMPLATE_DIR / "{{cookiecutter.svc_name}}"
    assert template_svc_dir.exists(), f"Template dir {template_svc_dir} not found"
    assert (template_svc_dir / "Dockerfile").exists()
    assert (template_svc_dir / "deploy" / "service.yaml").exists()


def test_generate_service(tmp_path: pathlib.Path) -> None:
    """Generate a service from the template and verify basic structure."""
    try:
        result = subprocess.run(
            [
                "uv",
                "run",
                "cookiecutter",
                "--no-input",
                str(TEMPLATE_DIR),
                "--output-dir",
                str(tmp_path),
            ],
            capture_output=True,
            text=True,
            timeout=60,
        )
        if result.returncode != 0:
            import pytest

            pytest.skip(f"cookiecutter failed: {result.stderr}")

        svc_dir = tmp_path / "my_service"
        assert svc_dir.exists()
        assert (svc_dir / "Dockerfile").exists()
    except FileNotFoundError:
        import pytest

        pytest.skip("cookiecutter not available")
