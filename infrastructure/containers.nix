{ config, pkgs, ... }:


let
  golemPath = "/var/lib/podman/volumes/golem";
  credentials = {
    registry = "build.shiku.world";
    username = "dockerreg";
    passwordFile = "/run/secrets/docker-registry.password";
  };
in
{
  imports = [
    ./ensure-paths.nix
  ];
  ensurePaths = [ golemPath ];

  secrets."docker-registry.password".file = ./secrets/docker-registry.password.age;
  virtualisation.oci-containers.containers = {
    "shiku-world-golem-bot" = {
      image = "build.shiku.world/golem-bot:latest";
      login = credentials;
      volumes = [
        "${golemPath}:/app/storage"
      ];
    };
  };
    virtualisation.oci-containers.containers = {
      "shiku-world-status" = {
        image = "build.shiku.world/shiku-world-status:latest";
        login = credentials;
        ports = ["3333:3000"];
      };
    };
}
