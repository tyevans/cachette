"""A thin client for the local vision model endpoint.

The endpoint speaks the OpenAI chat completion protocol. It needs no API
key. One module-level constant pins the model, so no caller chooses it.
"""

from __future__ import annotations

import base64
import json
import os
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field

# The model. Change it here and nowhere else.
MODEL = "qwen3.8-27b-16k"

# The context window of the model, in tokens.
CONTEXT_TOKENS = 16384

# The endpoint. An environment variable overrides it for a different host.
BASE_URL = os.environ.get("DIRECT_DIE_BASE_URL", "http://192.168.1.14:8080/v1")

# The default request timeout, in seconds.
TIMEOUT_SECONDS = float(os.environ.get("DIRECT_DIE_TIMEOUT", "300"))


class ClientError(RuntimeError):
    """The endpoint failed to give a usable answer."""


@dataclass
class Reply:
    """One answer from the model."""

    text: str
    prompt_tokens: int = 0
    completion_tokens: int = 0
    seconds: float = 0.0

    @property
    def total_tokens(self) -> int:
        """Give the sum of the prompt tokens and the completion tokens."""
        return self.prompt_tokens + self.completion_tokens


@dataclass
class Image:
    """One PNG image, and the label that tells the model what it is."""

    label: str
    data: bytes = field(repr=False)

    def data_url(self) -> str:
        """Give the image as a base64 data URL."""
        return "data:image/png;base64," + base64.b64encode(self.data).decode("ascii")


def _post(payload: dict, timeout: float) -> dict:
    """Send one request to the endpoint and give the decoded answer."""
    body = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        BASE_URL.rstrip("/") + "/chat/completions",
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as error:
        detail = error.read().decode("utf-8", "replace")[:400]
        raise ClientError(f"the endpoint returned HTTP {error.code}: {detail}") from error
    except urllib.error.URLError as error:
        raise ClientError(f"the endpoint is unreachable: {error.reason}") from error
    except TimeoutError as error:
        raise ClientError("the endpoint timed out") from error
    except json.JSONDecodeError as error:
        raise ClientError("the endpoint returned text that is not JSON") from error


def _complete(
    messages: list[dict],
    temperature: float,
    max_tokens: int,
    timeout: float,
    thinking: bool = False,
) -> Reply:
    """Run one chat completion and give the reply.

    The model thinks before it answers unless a caller stops it. A long
    thought fills the token budget and leaves the answer empty, so the
    default stops it.
    """
    payload = {
        "model": MODEL,
        "messages": messages,
        "temperature": temperature,
        "max_tokens": max_tokens,
        "stream": False,
    }
    if not thinking:
        payload["chat_template_kwargs"] = {"enable_thinking": False}
    started = time.monotonic()
    answer = _post(payload, timeout)
    seconds = time.monotonic() - started

    choices = answer.get("choices") or []
    if not choices:
        raise ClientError("the endpoint returned no choice")
    text = (choices[0].get("message") or {}).get("content")
    if not isinstance(text, str) or not text.strip():
        # A refusal or an empty answer must not stop the loop. The caller
        # decides what to do with an empty reply.
        text = ""
    usage = answer.get("usage") or {}
    return Reply(
        text=text,
        prompt_tokens=int(usage.get("prompt_tokens") or 0),
        completion_tokens=int(usage.get("completion_tokens") or 0),
        seconds=seconds,
    )


def ask(
    prompt: str,
    system: str | None = None,
    temperature: float = 0.7,
    max_tokens: int = 2600,
    timeout: float = TIMEOUT_SECONDS,
    thinking: bool = False,
) -> Reply:
    """Send text only, and give the reply."""
    messages: list[dict] = []
    if system:
        messages.append({"role": "system", "content": system})
    messages.append({"role": "user", "content": prompt})
    return _complete(messages, temperature, max_tokens, timeout, thinking)


def ask_with_images(
    prompt: str,
    images: list[Image],
    system: str | None = None,
    temperature: float = 0.4,
    max_tokens: int = 1200,
    timeout: float = TIMEOUT_SECONDS,
    thinking: bool = False,
) -> Reply:
    """Send text and one or more labelled images, and give the reply.

    Each label goes in before the image that follows it. The label tells
    the model what the image shows, because the model cannot see a file
    name.
    """
    parts: list[dict] = []
    for image in images:
        parts.append({"type": "text", "text": f"[{image.label}]"})
        parts.append({"type": "image_url", "image_url": {"url": image.data_url()}})
    parts.append({"type": "text", "text": prompt})

    messages: list[dict] = []
    if system:
        messages.append({"role": "system", "content": system})
    messages.append({"role": "user", "content": parts})
    return _complete(messages, temperature, max_tokens, timeout, thinking)
