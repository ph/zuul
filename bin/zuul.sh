#! /nix/store/9nw8b61s8lfdn8fkabxhbz0s775gjhbr-bash-5.2p37/bin/bash -e
LD_LIBRARY_PATH=${LD_LIBRARY_PATH:+':'$LD_LIBRARY_PATH':'}
LD_LIBRARY_PATH=${LD_LIBRARY_PATH/':''/nix/store/wcawg0k1fw6wih9g43pzf5i9h3hx357n-wayland-protocols-1.42/lib'':'/':'}
LD_LIBRARY_PATH='/nix/store/wcawg0k1fw6wih9g43pzf5i9h3hx357n-wayland-protocols-1.42/lib'$LD_LIBRARY_PATH
LD_LIBRARY_PATH=${LD_LIBRARY_PATH#':'}
LD_LIBRARY_PATH=${LD_LIBRARY_PATH%':'}
export LD_LIBRARY_PATH
LD_LIBRARY_PATH=${LD_LIBRARY_PATH:+':'$LD_LIBRARY_PATH':'}
LD_LIBRARY_PATH=${LD_LIBRARY_PATH/':''/nix/store/57k6cvcg2sl7dazypggn9vv5ip9xdac2-wayland-1.23.1/lib'':'/':'}
LD_LIBRARY_PATH='/nix/store/57k6cvcg2sl7dazypggn9vv5ip9xdac2-wayland-1.23.1/lib'$LD_LIBRARY_PATH
LD_LIBRARY_PATH=${LD_LIBRARY_PATH#':'}
LD_LIBRARY_PATH=${LD_LIBRARY_PATH%':'}
export LD_LIBRARY_PATH
exec -a "$0" "/home/ph/src/zuul/target/debug/zuul"  "$@" 
