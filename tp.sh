#!/bin/bash

function tp() {
    if [[ $# -eq 1 && "$1" != "ls" && "$1" != "help" && "$1" != "m" && "$1" != "bm" ]]
    then
        dir=$(teleport g $1)
        if [ "$dir" != "" ]
        then
            cd $dir
        fi
    else
        teleport "$@"
    fi
