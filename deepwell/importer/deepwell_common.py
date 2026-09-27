"""
Helper module to make sending DEEPWELL requests easier.
"""

from datetime import date, datetime
from typing import Any, NamedTuple, TypedDict

import requests

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
        super().__init__(error_data["message"])
        self.error_data = error_data


# Request-specific types
# NOTE: Any timestamps need to be timezone-aware


class UploadBlobData(NamedTuple):
    uploading_user_id: int
    buffer: bytes


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
    importing_user_id: int
    ip_address: str


# Main service class


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

    def upload_blob(self, request: UploadBlobData) -> str:
        # Start the upload
        output = self.request(
            "blob_upload",
            {
                "user_id": request.uploading_user_id,
                "blob_size": len(request.buffer),
            },
        )

        # Upload to the presign URL
        blob_id = output["pending_blob_id"]
        requests.put(output["presign_url"])

        # Return the blob ID to the user to finish the upload
        return blob_id

    def import_user(self, request: ImportUserData) -> int:
        match request.wikidot_user_type:
            case ImportExistingUser(name, slug):
                wikidot_user_type = {
                    "user_type": "extant",
                    "name": name,
                    "slug": slug,
                }
            case ImportDeletedUser():
                wikidot_user_type = {
                    "user_type": "deleted",
                }

        output = self.request(
            "import_wikidot_user",
            {
                "user_id": request.user_id,
                "created_at": request.created_at.isoformat(),
                "fetched_at": request.fetched_at.isoformat(),
                "wikidot_user_type": wikidot_user_type,
                "avatar_uploaded_blob_id": request.avatar_uploaded_blob_id,
                "real_name": request.real_name,
                "gender": request.gender,
                "birthday": request.birthday,
                "location": request.location,
                "biography": request.biography,
                "website": request.website,
                "karma": request.karma,
                "is_pro": request.is_pro,
                "importing_user_id": request.importing_user_id,
                "ip_address": request.ip_address,
            },
        )

        match output:
            case {"user_id": user_id}:
                assert user_id == request.user_id, "User ID doesn't match output"
                return user_id
            case _:
                raise ValueError(
                    f"Unexpected success output from import_wikidot_user: {output}"
                )
