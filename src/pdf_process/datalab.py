from __future__ import annotations

import base64
import os
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.parse import urljoin

import requests

API_BASE_URL = "https://www.datalab.to"
DEFAULT_TIMEOUT = 60


@dataclass(frozen=True)
class ConvertResult:
    html: str
    raw_response: dict[str, Any]


class DatalabError(RuntimeError):
    pass


def convert_pdf_to_html(
    input_pdf: Path,
    *,
    api_key: str | None = None,
    mode: str = "accurate",
    add_block_ids: bool = True,
    disable_image_extraction: bool = False,
    poll_interval: float = 2.0,
    timeout_seconds: float = 600.0,
    save_images: bool = True,
) -> ConvertResult:
    if not input_pdf.exists():
        raise FileNotFoundError(f"Input file not found: {input_pdf}")
    if input_pdf.suffix.lower() != ".pdf":
        raise ValueError("Input file must be a PDF.")

    resolved_api_key = api_key or os.environ.get("DATALAB_API_KEY")
    if not resolved_api_key:
        raise DatalabError("Missing Datalab API key. Set DATALAB_API_KEY or pass --api-key.")

    headers = {"X-API-Key": resolved_api_key}
    payload = {
        "mode": mode,
        "output_format": "html",
        "paginate": "false",
        "add_block_ids": _bool_form(add_block_ids),
        "disable_image_extraction": _bool_form(disable_image_extraction),
    }

    with input_pdf.open("rb") as file_handle:
        files = {"file": (input_pdf.name, file_handle, "application/pdf")}
        response = requests.post(
            f"{API_BASE_URL}/api/v1/convert",
            data=payload,
            files=files,
            headers=headers,
            timeout=DEFAULT_TIMEOUT,
        )
    initial = _json_response(response)
    _raise_for_api_error(initial, "Datalab convert request failed")

    check_url = initial.get("request_check_url") or f"{API_BASE_URL}/api/v1/convert/{initial.get('request_id')}"
    if not isinstance(check_url, str) or not check_url:
        raise DatalabError("Datalab response did not include a result check URL.")
    check_url = urljoin(API_BASE_URL, check_url)

    result = _poll_result(check_url, headers=headers, poll_interval=poll_interval, timeout_seconds=timeout_seconds)
    if not result.get("html") and result.get("result_url"):
        result = _download_result(str(result["result_url"]))

    html = result.get("html")
    if not isinstance(html, str) or not html.strip():
        raise DatalabError("Datalab conversion completed without HTML output.")

    if save_images:
        _save_images(input_pdf.parent, result.get("images"))

    return ConvertResult(html=html, raw_response=result)


def _poll_result(
    check_url: str,
    *,
    headers: dict[str, str],
    poll_interval: float,
    timeout_seconds: float,
) -> dict[str, Any]:
    deadline = time.monotonic() + timeout_seconds
    last_status = ""

    while time.monotonic() < deadline:
        response = requests.get(check_url, headers=headers, timeout=DEFAULT_TIMEOUT)
        result = _json_response(response)
        _raise_for_api_error(result, "Datalab conversion failed")

        status = str(result.get("status", "")).lower()
        last_status = status or last_status
        if status == "complete":
            return result
        if status in {"failed", "error", "cancelled", "canceled"}:
            raise DatalabError(result.get("error") or f"Datalab conversion ended with status: {status}")

        time.sleep(poll_interval)

    raise DatalabError(f"Timed out waiting for Datalab conversion to complete. Last status: {last_status or 'unknown'}")


def _json_response(response: requests.Response) -> dict[str, Any]:
    try:
        response.raise_for_status()
    except requests.HTTPError as error:
        raise DatalabError(f"HTTP {response.status_code}: {response.text}") from error
    try:
        data = response.json()
    except ValueError as error:
        raise DatalabError(f"Expected JSON response, got: {response.text[:200]}") from error
    if not isinstance(data, dict):
        raise DatalabError("Expected Datalab response to be a JSON object.")
    return data


def _download_result(result_url: str) -> dict[str, Any]:
    response = requests.get(result_url, timeout=DEFAULT_TIMEOUT)
    return _json_response(response)


def _raise_for_api_error(data: dict[str, Any], prefix: str) -> None:
    if data.get("success") is False:
        raise DatalabError(f"{prefix}: {data.get('error') or 'unknown error'}")
    if data.get("error"):
        raise DatalabError(f"{prefix}: {data['error']}")


def _bool_form(value: bool) -> str:
    return "true" if value else "false"


def _save_images(output_dir: Path, images: object) -> None:
    if not isinstance(images, dict):
        return

    for name, encoded in images.items():
        if not isinstance(name, str) or not isinstance(encoded, str):
            continue
        image_path = output_dir / name
        image_path.parent.mkdir(parents=True, exist_ok=True)
        image_path.write_bytes(base64.b64decode(encoded))
