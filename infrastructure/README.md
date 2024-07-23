v2202302194870220598.powersrv.de
user: shiku
pw: xTrgz0UnHx424OI

DOCKER_REGISTRY: build.shiku.world
DOCKER_REGISTRY_USERNAME: dockerreg
DOCKER_REGISTRY_PASSWORD: 5%5gq6zGX^i5phPTFPN8t

NIX_SSHOPTS='-i /home/shiku/.ssh/netcup/shikus-world-main/id_ed25519' nix run nixpkgs#nixos-rebuild --extra-experimental-features nix-command --extra-experimental-features flakes -- test --target-host root@v2202302194870220598.powersrv.de --flake .#neko

ssh -i /home/shiku/.ssh/netcup/shikus-world-main/id_ed25519 root@v2202302194870220598.powersrv.de

sudo sshfs -o allow_other,IdentityFile=/home/shiku/.ssh/netcup/shikus-world-main/id_ed25519 root@v2202302194870220598.powersrv.de:/var/lib/podman/volumes/shiku-world-home-dev-resources /home/shiku/RustroverProjects/shiku.world-project/shiku-world-home-out-mount/

nix flake update --extra-experimental-features nix-command --extra-experimental-features flakes

dependencies: nix-env -iA nixpkgs.yarn2nix

age1erawsedazwv782qc63nc5cljns5evsmjpla6kv87uqxjmcguz9mqzxlcj8
AGE-SECRET-KEY-16PJH5ND3UUG477E5JP2TUPWHC6NWQZXFRMWDKD0V4TLACNTA5D0Q9AZQ0T
