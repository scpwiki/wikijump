#!/bin/sh
curl \
	-H "X-Exempt-RateLimit: $RATE_LIMIT_SECRET" \
	-i http://localhost:2747/api/vI/ping
