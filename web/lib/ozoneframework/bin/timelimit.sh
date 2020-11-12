#!/bin/bash

TIME="$1" # in seconds

shift

"$@" &

TIME=$((TIME*2))
while [ $TIME -gt 0 ]; do
	TIME=$((TIME-1))
	sleep 0.5
	[ `jobs -r | wc -l` -eq 0 ] && exit
done

kill %1 2>/dev/null
sleep 1
kill -9 %1 2>/dev/null

