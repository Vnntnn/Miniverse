#!/bin/bash
# Test MQTT subscription
mosquitto_sub -h localhost -t "miniverse/#" -v
