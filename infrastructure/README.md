v2202302194870220598.powersrv.de
user: shiku
pw: xTrgz0UnHx424OI

NIX_SSHOPTS="-i ~/.ssh/netcup/id_rsa" nix run nixpkgs#nixos-rebuild -- test --target-host root@v2202302194870220598.powersrv.de --use-remote-sudo --flake .#neko


DOCKER_REGISTRY: build.shiku.world
DOCKER_REGISTRY_USERNAME: dockerreg
DOCKER_REGISTRY_PASSWORD: 5%5gq6zGX^i5phPTFPN8t

NIX_SSHOPTS='-i /home/shiku/.ssh/netcup/id_rsa'  nix run nixpkgs#nixos-rebuild -- test --target-host root@v2202302194870220598.powersrv.de --use-substitutes --use-remote-sudo --flake .#neko

dependencies: nix-env -iA nixpkgs.yarn2nix

age1erawsedazwv782qc63nc5cljns5evsmjpla6kv87uqxjmcguz9mqzxlcj8
AGE-SECRET-KEY-16PJH5ND3UUG477E5JP2TUPWHC6NWQZXFRMWDKD0V4TLACNTA5D0Q9AZQ0T
