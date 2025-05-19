#!/usr/bin/env bash

./image_to_video.sh -i ~/Downloads/Duplication/ -o duplication.mp4

if diff input.txt input.approved.txt > diff_output.txt; then
    echo "passed"
else
    cat diff_output.txt
fi
