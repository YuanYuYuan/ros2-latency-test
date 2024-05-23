#!/usr/bin/env bash


source install/setup.bash

PING="./target/release/ping"
PONG="./target/release/pong"

# # fastdds
# export RMW_IMPLEMENTATION=rmw_fastrtps_cpp
# export FASTRTPS_DEFAULT_PROFILES_FILE=$(realpath config/disable-fastdds-shm.xml)

# # cyclonedds
# export RMW_IMPLEMENTATION=rmw_cyclonedds_cpp
# export CYCLONEDDS_URI=file://$(realpath config/disable-cyclonedds-shm.xml)

# zenoh
export RMW_IMPLEMENTATION=rmw_zenoh_cpp
# export ZENOH_RUNTIME='(rx: (worker_threads: 1))'
# export ZENOH_RUNTIME='(rx: (handover: app), net: (handover: app), acc: (handover: app), app: (worker_threads: 1))'

echo "RMW: $RMW_IMPLEMENTATION"
parallel --halt now,success=1 --lb <<EOF
taskset -c 0,2 $PONG
sleep 1; taskset -c 1,3 $PING
EOF
