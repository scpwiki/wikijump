"""
Helper module to make sending DEEPWELL requests easier.
"""

from typing import Any, TypedDict

import requests


class DeepwellErrorData(TypedDict):
    call_trace: str
    code_trace: str
    extra: Any


class JsonrpcError(TypedDict):
    code: int
    message: str
    data: DeepwellErrorData | str | None


class DeepwellError(RuntimeError):
    def __init__(self, error_data: JsonrpcError):
        super().__init__(error_data["message"])
        self.error_data = error_data


class Deepwell:
    def __init__(self, host: str, port: int) -> None:
        self.host = host
        self.port = port
        self.endpoint = f"http://{host}:{port}/jsonrpc"

    def request(self, method: str, data: Any, id: int = 0) -> Any:
        r = requests.post(
            self.endpoint,
            json={
                "jsonrpc": "2.0",
                "method": method,
                "params": data,
                "id": id,
            },
        )
        r.raise_for_status()
        match r.json():
            case {"jsonrpc": "2.0", "id": id, "result": data}:
                return data
            case {"jsonrpc": "2.0", "id": id, "error": data}:
                raise DeepwellError(data)
            case data:
                raise ValueError(f"Unexpected JSONRPC response: {data}")
