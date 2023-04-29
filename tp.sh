#!/bin/bash

function tp() {
    if [[ $# -eq 1 && "$1" != "ls" ]]
    then
        dir=$(teleport g $1)
        if [ "$dir" != "" ]
        then
            cd $dir
        fi
    else
        teleport "$@"
    fi
}