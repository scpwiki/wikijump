"""
Helper module to make sending DEEPWELL requests easier.
"""

import logging
from collections.abc import Callable
from contextlib import contextmanager
from datetime import date, datetime
from functools import wraps
from typing import Any, NamedTuple, TypedDict, TypeVar

import requests
from urllib3.util import connection

logger = logging.getLogger()

T = TypeVar("T")
U = TypeVar("U")

# General JSONRPC / DEEPWELL types


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
        self.error_data = error_data
        super().__init__(self._message)

    @property
    def _message(self) -> str:
        extra_data = self.error_data["data"]
        if isinstance(extra_data, dict) and "call_trace" in extra_data:
            trace = extra_data["call_trace"]
            return f"Request failure (with trace):\n{trace}"

        # fallback
        return self.error_data["message"]


# Request-specific types
# NOTE: Any timestamps need to be timezone-aware


class UploadBlobData(NamedTuple):
    uploading_user_id: int
    blob: bytes
    mime_type: str


class ImportExistingUser(NamedTuple):
    name: str
    slug: str


class ImportDeletedUser(NamedTuple):
    pass


class ImportUserData(NamedTuple):
    user_id: int
    created_at: datetime
    fetched_at: datetime
    wikidot_user_type: ImportExistingUser | ImportDeletedUser
    avatar_uploaded_blob_id: str | None
    real_name: str | None
    gender: str | None
    birthday: date | None
    location: str | None
    biography: str | None
    website: str | None
    karma: int
    is_pro: bool
    upsert: bool
    importing_user_id: int
    ip_address: str


# Utilities


def map_null(value: T | None, callback: Callable[[T], U]) -> U | None:
    if value is None:
        return None

    return callback(value)


@contextmanager
def remap_connection(mapper: Callable[[str, int], tuple[str, int]]):
    original_create_connection = connection.create_connection

    @wraps(original_create_connection)
    def wrap_create_connection(address, *args, **kwargs):
        host, port = address
        host, port = mapper(host, port)
        return original_create_connection((host, port), *args, **kwargs)

    try:
        connection.create_connection = wrap_create_connection
        yield
    finally:
        connection.create_connection = original_create_connection


def remap_connection_for_s3():
    def map_local_s3(host: str, port: int) -> tuple[str, int]:
        # for local S3
        if host == "files":
            host = "localhost"

        return host, port

    return remap_connection(map_local_s3)


# Main service class


class Deepwell:
    def __init__(self, host: str, port: int) -> None:
        self.host = host
        self.port = port
        self.endpoint = f"http://{host}:{port}/jsonrpc"

    def request(self, method: str, data: Any, id: int = 0) -> Any:
        logger.debug("Sent request %s (ID %d), data %r", method, id, data)
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

    def upload_blob(self, request: UploadBlobData) -> str:
        # Start the upload
        logger.info("Requesting blob upload (%d bytes)", len(request.blob))
        output = self.request(
            "blob_upload",
            {
                "user_id": request.uploading_user_id,
                "blob_size": len(request.blob),
            },
        )

        # Upload to the presign URL
        blob_id = output["pending_blob_id"]
        presign_url = output["presign_url"]

        # Modify hostname to point to local S3 (same as --connect-to in curl)
        with remap_connection_for_s3():
            logger.info("Uploading blob ID %s", blob_id)
            logger.debug("Presign URL: %s", presign_url)
            r = requests.put(
                presign_url,
                data=request.blob,
                headers={"Content-Type": request.mime_type},
            )
            r.raise_for_status()
        logger.debug("Upload finished")

        # Return the blob ID to the user to finish the upload
        return blob_id

    # TODO: type the user output
    def get_user(self, id_or_slug: int | str) -> dict[str, Any] | None:
        logger.info("Fetching user %s", id_or_slug)
        return self.request("user_get", {"user": id_or_slug})

    def user_exists(self, id_or_slug: int | str) -> bool:
        return self.get_user(id_or_slug) is not None

    def import_user(self, request: ImportUserData, index: int | None = None) -> int:
        request_data = {
            "user_id": request.user_id,
            "created_at": request.created_at.isoformat(),
            "fetched_at": request.fetched_at.isoformat(),
            "avatar_uploaded_blob_id": request.avatar_uploaded_blob_id,
            "real_name": request.real_name,
            "gender": request.gender,
            "birthday": map_null(request.birthday, lambda date: date.isoformat()),
            "location": request.location,
            "biography": request.biography,
            "website": request.website,
            "karma": request.karma,
            "is_pro": request.is_pro,
            "upsert": request.upsert,
            "importing_user_id": request.importing_user_id,
            "ip_address": request.ip_address,
        }

        index_prefix = "" if index is None else f"[row {index}] "
        match request.wikidot_user_type:
            case ImportExistingUser(name, slug):
                logger.info(
                    "%sImporting user ID %d (%s, %r)",
                    index_prefix,
                    request.user_id,
                    name,
                    slug,
                )
                request_data.update(
                    user_type="extant",
                    name=name,
                    slug=slug,
                )
            case ImportDeletedUser():
                logger.info("%sImporting user ID %d (deleted)", index_prefix, request.user_id)
                request_data.update(user_type="deleted")

        logger.debug("Full request data: %r", request)

        output = self.request("import_wikidot_user", request_data)

        match output:
            case {"user_id": user_id}:
                assert user_id == request.user_id, "User ID doesn't match output"
                return user_id
            case _:
                raise ValueError(
                    f"Unexpected success output from import_wikidot_user: {output}"
                )
