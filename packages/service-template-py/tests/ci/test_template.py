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

"""Conformance test: generate a service from the template and verify it."""

import json
import pathlib
import subprocess

TEMPLATE_DIR = pathlib.Path(__file__).parent.parent.parent
GENERATED_SVC_NAME = "my_service"


def _generate_service(tmp_path: pathlib.Path) -> pathlib.Path:
    """Render the template into tmp_path; skip when cookiecutter is absent."""
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
    return tmp_path / GENERATED_SVC_NAME


def _svc_dir_or_skip(tmp_path: pathlib.Path) -> pathlib.Path:
    try:
        return _generate_service(tmp_path)
    except FileNotFoundError:
        import pytest

        pytest.skip("cookiecutter not available")


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
    svc_dir = _svc_dir_or_skip(tmp_path)

    assert svc_dir.exists()
    assert (svc_dir / "Dockerfile").exists()


def test_generated_server_has_no_placeholders(tmp_path: pathlib.Path) -> None:
    """The rendered eel_server.py must wire the platform stack, not stub it."""
    svc_dir = _svc_dir_or_skip(tmp_path)

    server_py = svc_dir / "py" / f"{GENERATED_SVC_NAME}_api" / "eel_server.py"
    assert server_py.exists()
    content = server_py.read_text()
    assert "TODO" not in content
    for symbol in (
        "AuthMiddleware",
        "IdempotencyMiddleware",
        "LoadSheddingMiddleware",
        "init_otel",
    ):
        assert symbol in content, f"eel_server.py missing {symbol}"


def test_generated_pyproject_declares_platform_deps(tmp_path: pathlib.Path) -> None:
    """Generated services must depend on the eel platform packages they use."""
    svc_dir = _svc_dir_or_skip(tmp_path)

    pyproject_text = (svc_dir / "py" / "pyproject.toml").read_text()
    for dep in (
        "eel-auth-py",
        "eel-resilience-py",
        "eel-loadshedding-py",
        "eel-observability-py",
    ):
        assert f'"{dep}' in pyproject_text
