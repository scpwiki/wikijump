#!/bin/sh

curl -i -X POST --json '{"jsonrpc":"2.0","method":"ping","id":0}' http://localhost:2747/jsonrpc
