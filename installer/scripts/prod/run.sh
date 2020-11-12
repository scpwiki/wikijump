#!/bin/bash
set -eu

source ./services.sh
trap stop_all SIGTERM SIGINT

# Starts the services then polls on them forever, only exiting if any of them fail

start_all
poll_all_forever
