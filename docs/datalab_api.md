> ## Documentation Index
> Fetch the complete documentation index at: https://documentation.datalab.to/llms.txt
> Use this file to discover all available pages before exploring further.

# API Overview

> REST API reference for document conversion, form filling, and file management.

Datalab provides REST APIs for document conversion, structured extraction, form filling, and file management. All APIs use the same authentication and follow similar patterns.

<Note>
  For the simplest integration, use the [Python SDK](/docs/welcome/sdk). The SDK handles authentication, polling, and provides typed responses.
</Note>

## Authentication

All requests require an API key in the `X-API-Key` header:

```bash theme={null}
curl -X POST https://www.datalab.to/api/v1/convert \
  -H "X-API-Key: YOUR_API_KEY" \
  -F "file=@document.pdf"
```

Get your API key from the [API Keys dashboard](https://www.datalab.to/app/keys).

## Request Pattern

All processing endpoints follow this pattern:

1. **Submit** a document for processing (returns immediately with a `request_id`)
2. **Poll** the status endpoint until processing completes
3. **Retrieve** results from the completed response

### Submit Request

```bash theme={null}
POST /api/v1/{endpoint}
```

Response:

```json theme={null}
{
  "success": true,
  "request_id": "abc123",
  "request_check_url": "https://www.datalab.to/api/v1/{endpoint}/abc123"
}
```

### Poll for Results

```bash theme={null}
GET /api/v1/{endpoint}/{request_id}
```

Response while processing:

```json theme={null}
{
  "status": "processing"
}
```

Response when complete:

```json theme={null}
{
  "status": "complete",
  "success": true,
  ...results...
}
```

<Warning>
  Results are deleted from Datalab servers one hour after processing completes. Retrieve your results promptly.
</Warning>

## Document Conversion

Convert documents to Markdown, HTML, JSON, or chunks.

**Endpoint:** `POST /api/v1/convert`

### Request

```python theme={null}
import requests

url = "https://www.datalab.to/api/v1/convert"
headers = {"X-API-Key": "YOUR_API_KEY"}

with open("document.pdf", "rb") as f:
    response = requests.post(
        url,
        files={"file": ("document.pdf", f, "application/pdf")},
        data={
            "output_format": "markdown",
            "mode": "balanced",
        },
        headers=headers
    )

data = response.json()
check_url = data["request_check_url"]
```

### Parameters

| Parameter                    | Type   | Default    | Description                                                                                                                                                                                                        |
| ---------------------------- | ------ | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `file`                       | file   | -          | Document file (multipart upload)                                                                                                                                                                                   |
| `file_url`                   | string | -          | URL to document (alternative to file upload)                                                                                                                                                                       |
| `output_format`              | string | `markdown` | Output format: `markdown`, `html`, `json`, `chunks`                                                                                                                                                                |
| `mode`                       | string | `fast`     | Processing mode: `fast`, `balanced`, `accurate`                                                                                                                                                                    |
| `max_pages`                  | int    | -          | Maximum pages to process                                                                                                                                                                                           |
| `page_range`                 | string | -          | Specific pages (e.g., `"0-5,10"`, 0-indexed). For spreadsheets, filters by sheet index.                                                                                                                            |
| `paginate`                   | bool   | `false`    | Add page delimiters to output                                                                                                                                                                                      |
| `skip_cache`                 | bool   | `false`    | Skip cached results                                                                                                                                                                                                |
| `disable_image_extraction`   | bool   | `false`    | Don't extract images                                                                                                                                                                                               |
| `disable_image_captions`     | bool   | `false`    | Don't generate image captions                                                                                                                                                                                      |
| `save_checkpoint`            | bool   | `false`    | Save checkpoint for reuse                                                                                                                                                                                          |
| `word_bboxes`                | bool   | `false`    | Predict per-word bounding boxes. Each word is inlined as `<span data-bbox="..." data-confidence="...">` in HTML output. Billed at \$0.30/1K pages.                                                                 |
| `extras`                     | string | -          | Comma-separated: `track_changes`, `chart_understanding`, `extract_links`, `table_cell_bboxes`, `list_item_bboxes`, `infographic`, `new_block_types`. (`table_row_bboxes` is deprecated — use `table_cell_bboxes`.) |
| `add_block_ids`              | bool   | `false`    | Add block IDs to HTML for citations                                                                                                                                                                                |
| `include_markdown_in_chunks` | bool   | `false`    | Include markdown content in chunks output                                                                                                                                                                          |
| `token_efficient_markdown`   | bool   | `false`    | Optimize markdown for LLM token efficiency                                                                                                                                                                         |
| `fence_synthetic_captions`   | bool   | `false`    | Wrap synthetic image captions in HTML comments                                                                                                                                                                     |
| `additional_config`          | string | -          | JSON with extra config options                                                                                                                                                                                     |
| `webhook_url`                | string | -          | Override webhook URL for this request                                                                                                                                                                              |

### Processing Modes

| Mode       | Description                                         |
| ---------- | --------------------------------------------------- |
| `fast`     | Lowest latency, good for simple documents (default) |
| `balanced` | Balance of speed and accuracy                       |
| `accurate` | Highest accuracy, best for complex layouts          |

### Response

Poll `request_check_url` until `status` is `complete`:

```python theme={null}
import time

while True:
    response = requests.get(check_url, headers=headers)
    result = response.json()

    if result["status"] == "complete":
        break
    time.sleep(2)

print(result["markdown"])
```

Response fields:

| Field                 | Type   | Description                              |
| --------------------- | ------ | ---------------------------------------- |
| `status`              | string | `processing`, `complete`, or `failed`    |
| `success`             | bool   | Whether conversion succeeded             |
| `markdown`            | string | Markdown output (if format is markdown)  |
| `html`                | string | HTML output (if format is html)          |
| `json`                | object | JSON output (if format is json)          |
| `chunks`              | object | Chunked output (if format is chunks)     |
| `images`              | object | Extracted images as `{filename: base64}` |
| `metadata`            | object | Document metadata                        |
| `page_count`          | int    | Number of pages processed                |
| `parse_quality_score` | float  | Quality score (0-5)                      |
| `cost_breakdown`      | object | Cost in cents                            |
| `error`               | string | Error message if failed                  |

<Note>
  For structured data extraction, see the [Extract endpoint](#structured-extraction). For document segmentation, see the [Segment endpoint](#document-segmentation).
</Note>

## Structured Extraction

Extract structured data from documents using a JSON schema.

**Endpoint:** `POST /api/v1/extract`

### Request

```python theme={null}
import requests
import json

headers = {"X-API-Key": "YOUR_API_KEY"}

schema = {
    "invoice_number": {"type": "string", "description": "Invoice ID"},
    "total": {"type": "number", "description": "Total amount"},
    "line_items": {
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "description": {"type": "string"},
                "amount": {"type": "number"}
            }
        }
    }
}

response = requests.post(
    "https://www.datalab.to/api/v1/extract",
    files={"file": ("invoice.pdf", open("invoice.pdf", "rb"), "application/pdf")},
    data={
        "page_schema": json.dumps(schema),
        "mode": "balanced"
    },
    headers=headers
)

data = response.json()
check_url = data["request_check_url"]
```

### Parameters

| Parameter         | Type   | Default    | Description                                                                                                                                            |
| ----------------- | ------ | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `file`            | file   | -          | Document file (multipart upload)                                                                                                                       |
| `file_url`        | string | -          | URL to document (alternative to file upload)                                                                                                           |
| `page_schema`     | string | -          | JSON schema defining the data to extract. Required unless `schema_id` is provided.                                                                     |
| `schema_id`       | string | -          | ID of a [saved extraction schema](/docs/recipes/structured-extraction/saved-schemas) (e.g. `sch_k8Hx9mP2nQ4v`). Mutually exclusive with `page_schema`. |
| `schema_version`  | int    | -          | Version of the saved schema to use. Only valid with `schema_id`; defaults to the latest version.                                                       |
| `checkpoint_id`   | string | -          | Checkpoint ID from a previous `/convert` call (with `save_checkpoint=true`). Skips re-parsing.                                                         |
| `mode`            | string | `fast`     | Processing mode: `fast`, `balanced`, `accurate`                                                                                                        |
| `output_format`   | string | `markdown` | Output format: `markdown`, `html`, `json`, `chunks`                                                                                                    |
| `max_pages`       | int    | -          | Maximum pages to process                                                                                                                               |
| `page_range`      | string | -          | Specific pages (e.g., `"0-5,10"`, 0-indexed). For spreadsheets, filters by sheet index.                                                                |
| `save_checkpoint` | bool   | `false`    | Save a checkpoint after processing for reuse with subsequent calls                                                                                     |
| `webhook_url`     | string | -          | Override webhook URL for this request                                                                                                                  |

The extracted data is returned in `extraction_schema_json` in the poll response.

See [Structured Extraction](/docs/recipes/structured-extraction/api-overview) for detailed examples.

## Document Segmentation

Segment documents into structured sections using a JSON schema.

**Endpoint:** `POST /api/v1/segment`

### Parameters

| Parameter             | Type   | Default      | Description                                                                                    |
| --------------------- | ------ | ------------ | ---------------------------------------------------------------------------------------------- |
| `file`                | file   | -            | Document file (multipart upload)                                                               |
| `file_url`            | string | -            | URL to document (alternative to file upload)                                                   |
| `segmentation_schema` | string | **required** | JSON schema defining the segments to extract                                                   |
| `checkpoint_id`       | string | -            | Checkpoint ID from a previous `/convert` call (with `save_checkpoint=true`). Skips re-parsing. |
| `mode`                | string | `fast`       | Processing mode: `fast`, `balanced`, `accurate`                                                |

See [Document Segmentation](/docs/recipes/document-segmentation/auto-segmentation) for detailed examples.

## Track Changes

Extract tracked changes (insertions and deletions) from DOCX files, PDFs, and images.

**Endpoint:** `POST /api/v1/track-changes`

DOCX files use exact pandoc-based change extraction. PDFs and images are processed by a visual redline model.

```python theme={null}
response = requests.post(
    "https://www.datalab.to/api/v1/track-changes",
    files={"file": ("contract.pdf", open("contract.pdf", "rb"), "application/pdf")},
    headers=headers
)
```

See [Track Changes](/docs/recipes/extract-redlines-and-comments/track-changes-from-word-documents) for detailed examples.

## Custom Processor (Agent)

Execute custom AI-powered processors on documents.

**Endpoint:** `POST /api/v1/agent`

<Warning>
  `POST /api/v1/custom-processor` and `POST /api/v1/custom-pipeline` are deprecated (sunset: **October 31, 2026**). Migrate to `POST /api/v1/agent` — pass `processor=cp_xxx` instead of `pipeline_id=cp_xxx`.
</Warning>

### Parameters

| Parameter       | Type   | Default      | Description                                         |
| --------------- | ------ | ------------ | --------------------------------------------------- |
| `file`          | file   | -            | Document file (multipart upload)                    |
| `file_url`      | string | -            | URL to document                                     |
| `processor`     | string | **required** | Custom processor ID (`cp_XXXXX`)                    |
| `version`       | int    | -            | Processor version to run (default: active version)  |
| `run_eval`      | bool   | `false`      | Run evaluation rules defined for the processor      |
| `mode`          | string | `fast`       | Processing mode: `fast`, `balanced`, `accurate`     |
| `output_format` | string | `markdown`   | Output format: `markdown`, `html`, `json`, `chunks` |
| `webhook_url`   | string | -            | URL to POST when complete                           |

## Form Filling

Fill forms in PDFs and images.

**Endpoint:** `POST /api/v1/fill`

### Request

```python theme={null}
import json

field_data = {
    "full_name": {"value": "John Doe", "description": "Full legal name"},
    "date": {"value": "2024-01-15", "description": "Today's date"},
    "signature": {"value": "John Doe", "description": "Signature field"}
}

response = requests.post(
    "https://www.datalab.to/api/v1/fill",
    files={"file": ("form.pdf", open("form.pdf", "rb"), "application/pdf")},
    data={
        "field_data": json.dumps(field_data),
        "confidence_threshold": "0.5"
    },
    headers=headers
)
```

### Parameters

| Parameter              | Type   | Default | Description                           |
| ---------------------- | ------ | ------- | ------------------------------------- |
| `file`                 | file   | -       | Form file (PDF or image)              |
| `file_url`             | string | -       | URL to form                           |
| `field_data`           | string | -       | JSON mapping field names to values    |
| `context`              | string | -       | Additional context for field matching |
| `confidence_threshold` | float  | `0.5`   | Minimum confidence for matching (0-1) |
| `page_range`           | string | -       | Specific pages to process             |
| `skip_cache`           | bool   | `false` | Skip cached results                   |

### Field Data Format

```json theme={null}
{
  "field_key": {
    "value": "The value to fill",
    "description": "Description to help match the field"
  }
}
```

### Response

| Field              | Type   | Description                     |
| ------------------ | ------ | ------------------------------- |
| `status`           | string | Processing status               |
| `success`          | bool   | Whether filling succeeded       |
| `output_format`    | string | `pdf` or `png`                  |
| `output_base64`    | string | Base64-encoded filled form      |
| `fields_filled`    | array  | Successfully filled field names |
| `fields_not_found` | array  | Unmatched field names           |
| `page_count`       | int    | Pages processed                 |
| `cost_breakdown`   | object | Cost details                    |

See [Form Filling](/docs/recipes/form-filling/form-filling-api-overview) for more examples.

## File Management

Upload and manage files for use in pipelines.

### Upload File

**Step 1:** Request an upload URL

```bash theme={null}
POST /api/v1/files/upload
Content-Type: application/json

{
  "filename": "document.pdf",
  "content_type": "application/pdf"
}
```

Response:

```json theme={null}
{
  "file_id": 123,
  "upload_url": "https://...",
  "reference": "datalab://file-abc123"
}
```

**Step 2:** Upload directly to the presigned URL

```bash theme={null}
PUT {upload_url}
Content-Type: application/pdf

<file contents>
```

**Step 3:** Confirm upload

```bash theme={null}
GET /api/v1/files/{file_id}/confirm
```

### List Files

```bash theme={null}
GET /api/v1/files?limit=50&offset=0
```

### Get File Metadata

```bash theme={null}
GET /api/v1/files/{file_id}
```

### Get Download URL

```bash theme={null}
GET /api/v1/files/{file_id}/download?expires_in=3600
```

### Delete File

```bash theme={null}
DELETE /api/v1/files/{file_id}
```

See [File Management](/docs/recipes/file-management/file-upload-api) for detailed examples.

## Thumbnails

Generate page thumbnails from a previously processed document:

```bash theme={null}
GET /api/v1/thumbnails/{lookup_key}?thumb_width=300&page_range=0-2
```

| Parameter     | Type   | Default   | Description                               |
| ------------- | ------ | --------- | ----------------------------------------- |
| `lookup_key`  | string | Required  | The request ID from a previous conversion |
| `thumb_width` | int    | 300       | Thumbnail width in pixels                 |
| `page_range`  | string | All pages | Pages to generate (e.g., `"0,2-4"`)       |

Response:

```json theme={null}
{
  "success": true,
  "thumbnails": ["base64_encoded_jpg_1", "base64_encoded_jpg_2"]
}
```

Thumbnails are returned as base64-encoded JPG images.

## Create Document

Generate DOCX files from markdown with track changes support:

```bash theme={null}
POST /api/v1/create-document
Content-Type: application/json

{
  "markdown": "# Title\n\nThis is <ins data-revision-author=\"Editor\">newly added</ins> text.",
  "output_format": "docx"
}
```

See [Create Document](/docs/recipes/create-document/create-document-api-overview) for detailed examples.

## Webhooks

Configure webhooks to receive notifications when processing completes instead of polling.

Set a default webhook URL in your [account settings](https://www.datalab.to/settings), or override per-request with the `webhook_url` parameter.

See [Webhooks](/platform/webhooks) for configuration details.

## Rate Limits

Default rate limits apply per API key. If you exceed limits, you'll receive a `429` response.

See [Rate Limits](/docs/common/limits) for details and how to request higher limits.

## Next Steps

<CardGroup cols={2}>
  <Card title="SDK Reference" icon="code" href="/docs/welcome/sdk">
    Use the Python SDK for a simpler integration with typed responses.
  </Card>

  <Card title="Webhooks" icon="bell" href="/platform/webhooks">
    Receive notifications when processing completes instead of polling.
  </Card>

  <Card title="API Limits" icon="gauge" href="/docs/common/limits">
    Understand file size limits, page limits, and rate limiting.
  </Card>

  <Card title="Document Conversion" icon="file-lines" href="/docs/recipes/conversion/conversion-api-overview">
    Detailed guide to converting documents to Markdown, HTML, or JSON.
  </Card>
</CardGroup>


> ## Documentation Index
> Fetch the complete documentation index at: https://documentation.datalab.to/llms.txt
> Use this file to discover all available pages before exploring further.

# Document Conversion

> Convert documents to Markdown, HTML, JSON, or chunks using the Convert API.

Convert PDFs, Word documents, spreadsheets, and images to machine-readable formats. Marker handles complex layouts, tables, math, and images.

**Before you begin**, make sure you have:

1. A [Datalab account](https://www.datalab.to/auth/sign_up) with an [API key](https://www.datalab.to/app/keys) (new accounts include \$5 in free credits)
2. Python 3.10+ installed
3. The Datalab SDK: `pip install datalab-python-sdk`
4. Your `DATALAB_API_KEY` environment variable set

<Info>
  **Building for production?** Use [Pipelines](/docs/recipes/pipelines/pipeline-overview) to chain processors, version your configuration, and deploy with a single API call.
</Info>

## Quick Start

<CodeGroup>
  ```python Python SDK theme={null}
  from datalab_sdk import DatalabClient, ConvertOptions

  client = DatalabClient()

  # Basic conversion
  result = client.convert("document.pdf")
  print(result.markdown)

  # With options
  options = ConvertOptions(
      output_format="markdown",
      mode="balanced",
      paginate=True
  )
  result = client.convert("document.pdf", options=options)
  ```

  ```bash cURL theme={null}
  # Submit request
  curl -X POST https://www.datalab.to/api/v1/convert \
    -H "X-API-Key: $DATALAB_API_KEY" \
    -F "file=@document.pdf" \
    -F "output_format=markdown" \
    -F "mode=balanced"

  # Poll for results (use request_check_url from response)
  curl https://www.datalab.to/api/v1/convert/REQUEST_ID \
    -H "X-API-Key: $DATALAB_API_KEY"
  ```

  ```python Python (requests) theme={null}
  import os, time, requests

  API_URL = "https://www.datalab.to/api/v1/convert"
  headers = {"X-API-Key": os.getenv("DATALAB_API_KEY")}

  # Submit request
  with open("document.pdf", "rb") as f:
      response = requests.post(
          API_URL,
          files={"file": ("document.pdf", f, "application/pdf")},
          data={"output_format": "markdown", "mode": "balanced"},
          headers=headers
      )

  check_url = response.json()["request_check_url"]

  # Poll for completion
  for _ in range(300):
      result = requests.get(check_url, headers=headers).json()
      if result["status"] == "complete":
          print(result["markdown"])
          break
      time.sleep(2)
  ```
</CodeGroup>

The SDK handles polling automatically. For the REST API, you submit a request and poll the `request_check_url` until the status is `complete`.

See [SDK Conversion](/docs/welcome/sdk/conversion) for complete SDK documentation.

<Info>
  **File limits:** Maximum file size is 200 MB, with up to 7,000 pages per request. See [API Limits](/docs/common/limits) for the full list.
</Info>

## Parameters

### Core Parameters

| Parameter       | Type   | Default    | Description                                         |
| --------------- | ------ | ---------- | --------------------------------------------------- |
| `file`          | file   | -          | Document file (multipart upload)                    |
| `file_url`      | string | -          | URL to document (alternative to file)               |
| `output_format` | string | `markdown` | Output format: `markdown`, `html`, `json`, `chunks` |
| `mode`          | string | `fast`     | Processing mode (see below)                         |

<Tip>
  **Which output format should I use?**

  * **LLM/RAG pipelines** → `markdown` (default, most compatible)
  * **Web display** → `html` (preserves visual structure)
  * **Programmatic access to blocks** → `json` (includes bounding boxes and block types)
  * **Embedding and search** → `chunks` (pre-chunked for vector databases)
</Tip>

### Processing Modes

| Mode       | Description                                     | Best For                                         |
| ---------- | ----------------------------------------------- | ------------------------------------------------ |
| `fast`     | Lowest latency, good for simple documents       | High-throughput pipelines, simple layouts        |
| `balanced` | Balance of speed and accuracy **(recommended)** | Most use cases                                   |
| `accurate` | Highest accuracy, best for complex layouts      | Complex tables, dense layouts, scanned documents |

<Tip>
  **Which mode should I use?**

  * **Most use cases** → `balanced` (recommended default)
  * **Simple, clean PDFs** at high throughput → `fast`
  * **Scanned documents, complex tables, or dense layouts** → `accurate`
</Tip>

### Page Control

| Parameter    | Type   | Default | Description                                                                             |
| ------------ | ------ | ------- | --------------------------------------------------------------------------------------- |
| `max_pages`  | int    | -       | Maximum pages to process                                                                |
| `page_range` | string | -       | Specific pages (e.g., `"0-5,10"`, 0-indexed). For spreadsheets, filters by sheet index. |
| `paginate`   | bool   | `false` | Add page delimiters to output                                                           |

### Image Handling

| Parameter                  | Type | Default | Description                   |
| -------------------------- | ---- | ------- | ----------------------------- |
| `disable_image_extraction` | bool | `false` | Don't extract images          |
| `disable_image_captions`   | bool | `false` | Don't generate image captions |

### Advanced Options

| Parameter                    | Type   | Default | Description                                                                                                                                                                                                                |
| ---------------------------- | ------ | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `add_block_ids`              | bool   | `false` | Add `data-block-id` attributes to HTML elements                                                                                                                                                                            |
| `skip_cache`                 | bool   | `false` | Skip cached results                                                                                                                                                                                                        |
| `save_checkpoint`            | bool   | `false` | Save checkpoint for reuse                                                                                                                                                                                                  |
| `word_bboxes`                | bool   | `false` | Predict per-word bounding boxes with confidence scores. Each word is inlined into HTML output as a `<span data-bbox="..." data-confidence="...">` element (markdown output strips these). Billed at \$0.30 per 1K pages.   |
| `extras`                     | string | -       | Comma-separated: `track_changes`, `chart_understanding`, `extract_links`, `table_cell_bboxes`, `list_item_bboxes`, `infographic`, `new_block_types`. (`table_row_bboxes` is deprecated — use `table_cell_bboxes` instead.) |
| `include_markdown_in_chunks` | bool   | `false` | Include markdown content in chunks/JSON output                                                                                                                                                                             |
| `token_efficient_markdown`   | bool   | `false` | Optimize markdown for LLM token efficiency                                                                                                                                                                                 |
| `fence_synthetic_captions`   | bool   | `false` | Wrap synthetic image captions in HTML comments                                                                                                                                                                             |
| `additional_config`          | string | -       | JSON with extra config (see below)                                                                                                                                                                                         |
| `webhook_url`                | string | -       | Override webhook URL for this request                                                                                                                                                                                      |
| `processing_location`        | string | -       | Data residency region override: `"eu"` or `"us"`. When set, use `file_url` or a pre-uploaded `datalab://` reference — multipart uploads are not supported. EU processing carries a regional pricing premium.               |

<Note>
  For structured extraction, use the [Extract API](/docs/recipes/structured-extraction/api-overview). For document segmentation, use the [Segment API](/docs/recipes/document-segmentation/auto-segmentation).
</Note>

<Note>
  The `track_changes` extra is supported on this endpoint. You can also use the dedicated [Track Changes endpoint](/docs/recipes/extract-redlines-and-comments/track-changes-from-word-documents).
</Note>

### Bounding Box Add-ons

Three add-ons annotate HTML output with spatial coordinates and confidence scores. All are billed at **\$0.30 per 1K pages** each (additive on top of the base conversion rate) and require the `html` output format to expose the attributes.

| Add-on            | How to enable                | What it annotates                                                                       |
| ----------------- | ---------------------------- | --------------------------------------------------------------------------------------- |
| Word bboxes       | `word_bboxes=True`           | Every word in the document gets a `data-bbox` and `data-confidence` span in HTML        |
| Table cell bboxes | `extras="table_cell_bboxes"` | `<colgroup><col>`, `<tr>`, and `<td>`/`<th>` elements get `data-bbox`/`data-confidence` |
| List item bboxes  | `extras="list_item_bboxes"`  | Each `<li>` element gets `data-bbox`/`data-confidence`                                  |

<Warning>
  `table_cell_bboxes` and `list_item_bboxes` do **not** automatically enable `word_bboxes`. To also receive per-word bounding boxes, set `word_bboxes=True` explicitly alongside the structural extra. Each add-on is billed independently.
</Warning>

```python theme={null}
from datalab_sdk import DatalabClient, ConvertOptions

client = DatalabClient()

# Get table cell bboxes only
options = ConvertOptions(
    output_format="html",
    extras="table_cell_bboxes,list_item_bboxes",
)
result = client.convert("document.pdf", options=options)
# HTML contains data-bbox and data-confidence on table cells and list items

# To also get per-word bounding boxes, pass word_bboxes=True to the REST API
# (billed separately at $0.30/1K pages)
```

### Additional Config Options

Pass as JSON string in `additional_config`:

| Key                           | Type | Description                     |
| ----------------------------- | ---- | ------------------------------- |
| `keep_spreadsheet_formatting` | bool | Preserve spreadsheet formatting |
| `keep_pageheader_in_output`   | bool | Include page headers            |
| `keep_pagefooter_in_output`   | bool | Include page footers            |

Example:

```python theme={null}
options = ConvertOptions(
    additional_config={
        "keep_spreadsheet_formatting": True,
        "keep_pageheader_in_output": False
    }
)
```

## Response Fields

| Field                 | Type   | Description                                   |
| --------------------- | ------ | --------------------------------------------- |
| `status`              | string | `processing`, `complete`, or `failed`         |
| `success`             | bool   | Whether conversion succeeded                  |
| `output_format`       | string | Requested output format                       |
| `markdown`            | string | Markdown output (if format is markdown)       |
| `html`                | string | HTML output (if format is html)               |
| `json`                | object | JSON output (if format is json)               |
| `chunks`              | object | Chunked output (if format is chunks)          |
| `images`              | object | Extracted images as `{filename: base64}`      |
| `metadata`            | object | Document metadata                             |
| `page_count`          | int    | Number of pages processed                     |
| `parse_quality_score` | float  | Quality score (0-5)                           |
| `cost_breakdown`      | object | Cost in cents                                 |
| `checkpoint_id`       | string | Checkpoint ID (if `save_checkpoint` was true) |
| `error`               | string | Error message if failed                       |

## Examples

### Convert with High Accuracy

```python theme={null}
from datalab_sdk import DatalabClient, ConvertOptions

client = DatalabClient()

options = ConvertOptions(
    mode="accurate",
    output_format="markdown"
)

result = client.convert("complex_document.pdf", options=options)
print(f"Quality score: {result.parse_quality_score}")
print(result.markdown)
```

### HTML with Block IDs for Citations

```python theme={null}
options = ConvertOptions(
    output_format="html",
    add_block_ids=True
)

result = client.convert("document.pdf", options=options)
# HTML elements have data-block-id attributes for citation tracking
```

### Process Specific Pages

```python theme={null}
options = ConvertOptions(
    page_range="0-4,10,15-20",  # Pages 0-4, 10, and 15-20
    output_format="markdown"
)

result = client.convert("large_document.pdf", options=options)
```

### Process Specific Sheets from a Spreadsheet

For spreadsheet files, `page_range` filters by sheet index (0-based):

```python theme={null}
options = ConvertOptions(
    page_range="0,2",  # First and third sheets only
    output_format="markdown"
)

result = client.convert("workbook.xlsx", options=options)
```

### Extract Track Changes

```python theme={null}
options = ConvertOptions(
    extras="track_changes",
    output_format="json"
)

result = client.convert("document_with_changes.docx", options=options)
```

## Parse Quality Score

Every conversion response includes a `parse_quality_score` (0-5) that indicates how well the document was parsed:

| Score Range | Quality   | Recommended Action                                 |
| ----------- | --------- | -------------------------------------------------- |
| 4.0 - 5.0   | Excellent | Use the output directly                            |
| 3.0 - 3.9   | Good      | Review for minor issues                            |
| 2.0 - 2.9   | Fair      | Consider retrying with `accurate` mode             |
| 0.0 - 1.9   | Poor      | Retry with `accurate` mode or check the input file |

Use quality scores to build automated quality gates:

```python theme={null}
result = client.convert("document.pdf", options=ConvertOptions(mode="balanced"))

if result.parse_quality_score < 3.0:
    # Retry with higher accuracy
    result = client.convert("document.pdf", options=ConvertOptions(mode="accurate"))
```

Use quality scores to gate pipeline execution or route documents to different processing configurations.

## Checkpoints

Save a processing checkpoint to reuse parsed results for extraction or segmentation without re-processing:

```python theme={null}
from datalab_sdk import DatalabClient, ConvertOptions, ExtractOptions
import json

client = DatalabClient()

# Step 1: Convert and save checkpoint
options = ConvertOptions(
    save_checkpoint=True,
    output_format="markdown"
)
result = client.convert("document.pdf", options=options)
checkpoint_id = result.checkpoint_id

# Step 2: Use checkpoint for extraction (no re-processing needed)
extraction_options = ExtractOptions(
    page_schema=json.dumps({"type": "object", "properties": {"title": {"type": "string"}}}),
    checkpoint_id=checkpoint_id
)
extract_result = client.extract("document.pdf", options=extraction_options)
```

Checkpoints save time and cost when you need to run multiple operations (extraction, segmentation) on the same document.

<Warning>
  Results are deleted from Datalab servers one hour after processing completes. Retrieve your results promptly.
</Warning>

## Next Steps

<CardGroup cols={2}>
  <Card title="Structured Extraction" icon="table" href="/docs/recipes/structured-extraction/api-overview">
    Extract structured data from documents using JSON schemas
  </Card>

  <Card title="Batch Processing" icon="layer-group" href="/docs/recipes/conversion/batch-documents">
    Process multiple documents concurrently
  </Card>

  <Card title="Document Segmentation" icon="scissors" href="/docs/recipes/document-segmentation/auto-segmentation">
    Split multi-document PDFs into segments
  </Card>

  <Card title="Webhooks" icon="bell" href="/platform/webhooks">
    Get notified when conversions complete
  </Card>
</CardGroup>
