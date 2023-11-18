{ config, pkgs, ... }:


let
  golemPath = "/var/lib/podman/volumes/golem";
  shikuWorldResourcesPath = "/var/lib/podman/volumes/shiku-world-resources";
  shikuWorldResourcesConfigPath = "/var/lib/podman/volumes/shiku-world-resources-config";
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
  ensurePaths = [ golemPath shikuWorldResourcesPath shikuWorldResourcesConfigPath ];

  secrets."docker-registry.password".file = ./secrets/docker-registry.password.age;
  virtualisation.oci-containers.containers = {
    "shiku-world-golem-bot" = {
      image = "build.shiku.world/golem-bot:latest";
      login = credentials;
      volumes = [
        "${golemPath}:/app/storage"
      ];
    };
    "shiku-world-status" = {
      image = "build.shiku.world/shiku-world-status:latest";
      login = credentials;
      ports = ["3333:3000"];
    };
    "shiku-world-resources" = {
      image = "build.shiku.world/shiku-world-resources:latest";
      login = credentials;
      volumes = [
        "${shikuWorldResourcesPath}:/static"
      ];
      ports = ["8083:8083"];
    };
    "shiku-world-files" = {
      image = "hurlenko/filebrowser";
      login = credentials;
      volumes = [
        "${shikuWorldResourcesPath}:/data"
        "${shikuWorldResourcesConfigPath}:/config"
      ];
      ports = ["8088:8080"];
    };
  };
}
