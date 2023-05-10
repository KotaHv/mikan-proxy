#!/bin/sh
exec /mikan-proxy
if [ "${DEBUG}" = "true" ]; then
    while true
    do
        echo "exec /mikan-proxy fail"
        sleep 1000
    done
fi