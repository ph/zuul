#!/bin/sh

# SPDX-FileCopyrightText: 2025 Pier-Hugues Pellerin <ph@heykimo.com>
#
# SPDX-License-Identifier: MIT

export LD_LIBRARY_PATH=/gnu/store/sfbwscz1sibpr3b447rsw1vz1axsz9pp-profile/lib 
export CURRENT=/home/ph/src/zuul/target/debug/zuul

touch /tmp/OOK_sync
exec $CURRENT "$@"
