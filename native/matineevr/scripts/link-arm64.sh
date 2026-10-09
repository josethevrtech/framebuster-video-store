#!/bin/sh
for argument do
    shift
    case "$argument" in
        -Wl,-O1) ;;
        *) set -- "$@" "$argument" ;;
    esac
done
exec zig cc -target aarch64-linux-gnu.2.31 "$@"
