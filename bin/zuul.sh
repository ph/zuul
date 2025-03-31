set -e
export LD_LIBRARY_PATH=/gnu/store/sfbwscz1sibpr3b447rsw1vz1axsz9pp-profile/lib 

export CURRENT=$(dirname $0)/../target/debug/zuul

$CURRENT
