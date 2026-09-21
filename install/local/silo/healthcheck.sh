#!/bin/bash
set -e
port_number="${1:-9000}"  # pass in or default
timeout 5s bash -c ":> /dev/tcp/127.0.0.1/$port_number"
