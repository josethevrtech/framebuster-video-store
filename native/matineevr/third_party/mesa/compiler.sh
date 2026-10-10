#!/bin/sh
for argument do
    shift
    case "$argument" in
        -Wl,-O1) ;;
        csrDT) set -- "$@" csrD ;;
        *) set -- "$@" "$argument" ;;
    esac
done
exec zig "$@"
