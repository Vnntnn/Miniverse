#!/bin/bash
# Test MQTT publishing
mosquitto_pub -h localhost -t "miniverse/test" -m "Hello from terminal"
