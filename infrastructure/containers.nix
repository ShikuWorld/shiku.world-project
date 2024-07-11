{ config, pkgs, ... }:

let
  golemPath = "/var/lib/podman/volumes/golem";
  shikuWorldResourcesPath = "/var/lib/podman/volumes/shiku-world-resources";
  shikuWorldHomeDbDevPath = "/var/lib/podman/volumes/shiku-world-home-db-dev";
  shikuWorldHomeDevResourcePath = "/var/lib/podman/volumes/shiku-world-home-dev-resources";
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
  ensurePaths = [ golemPath shikuWorldResourcesPath shikuWorldResourcesConfigPath shikuWorldHomeDbDevPath shikuWorldHomeDevResourcePath ];
  systemd.services.create-shikus-world-network = with config.virtualisation.oci-containers; {
    serviceConfig.Type = "oneshot";
    wantedBy = [ "podman-shiku-world-home-dev.service" "podman-shiku-world-home-dev-db.service" ];
    script = ''
      ${pkgs.podman}/bin/podman network exists shiku-dev-net || \
      ${pkgs.podman}/bin/podman network create shiku-dev-net
      '';
  };
  secrets."docker-registry.password".file = ./secrets/docker-registry.password.age;
  secrets."shiku-world-home-dev-db-credentials".file = ./secrets/shiku-world-home-dev-db-credentials.age;
  virtualisation.oci-containers.containers = {
    "shiku-world-golem-bot" = {
      image = "build.shiku.world/golem-bot:latest";
      login = credentials;
      volumes = [
        "${golemPath}:/app/storage"
      ];
      extraOptions = [ "--pull=always" ];
    };
    "shiku-world-home-dev" = {
      image = "build.shiku.world/shiku-world-home-dev:0.2.2";
      login = credentials;
      ports = ["9001:9001" "3030:3030"];
      volumes = [
        "${shikuWorldHomeDevResourcePath}:/app/target/release/out"
      ];
      extraOptions = [ "--pull=always" "--network=shiku-dev-net" ];
    };
    "shiku-world-home-dev-db" = {
      image = "postgres";
      volumes = [
        "${shikuWorldHomeDbDevPath}:/var/lib/mysql"
      ];
      extraOptions = [ "--network=shiku-dev-net" ];
      environmentFiles = ["/run/secrets/shiku-world-home-dev-db-credentials"];
    };
    "shiku-world-status" = {
      image = "build.shiku.world/shiku-world-status:latest";
      login = credentials;
      ports = ["3333:3000"];
      extraOptions = [ "--pull=always" ];
    };
    "shiku-world-resources" = {
      image = "build.shiku.world/shiku-world-resources:latest";
      login = credentials;
      volumes = [
        "${shikuWorldResourcesPath}:/static"
      ];
      ports = ["8083:8083"];
      extraOptions = [ "--pull=always" ];
    };
    "shiku-world-files" = {
      image = "hurlenko/filebrowser";
      login = credentials;
      volumes = [
        "${shikuWorldResourcesPath}:/data"
        "${shikuWorldResourcesConfigPath}:/config"
      ];
      ports = ["8088:8080"];
      extraOptions = [ "--pull=always" ];
    };
  };
}
