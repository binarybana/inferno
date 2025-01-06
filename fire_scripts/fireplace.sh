#!/bin/bash

sudo $HOME/fire_scripts/headless --mpv-socket=/tmp/mpvsocket &> /tmp/headless.log &
mpv --fs --input-ipc-server=/tmp/mpvsocket --loop-file $HOME/fire_videos/*.webm
