{ config, pkgs, ... }:

let
  golemPath = "/var/lib/podman/volumes/golem";
  shikuWorldResourcesPath = "/var/lib/podman/volumes/shiku-world-resources";
  shikuWorldHomeDbDevPath = "/var/lib/podman/volumes/shiku-world-home-db-dev";
  shikuWorldHomeDevResourcePath = "/var/lib/podman/volumes/shiku-world-home-dev-resources";
  shikuWorldResourcesConfigPath = "/var/lib/podman/volumes/shiku-world-resources-config";
  registryDataPath = "/var/lib/podman/volumes/docker-registry-data";
  credentials = {
    registry = "dreg.shiku.world";
    username = "dockerreg";
    passwordFile = "/run/secrets/docker-registry.password";
  };
in
{
  imports = [
    ./ensure-paths.nix
  ];
  ensurePaths = [ golemPath shikuWorldResourcesPath shikuWorldResourcesConfigPath shikuWorldHomeDbDevPath shikuWorldHomeDevResourcePath registryDataPath ];
  systemd.services.create-shikus-world-network = with config.virtualisation.oci-containers; {
    serviceConfig.Type = "oneshot";
    wantedBy = [ "podman-shiku-world-home-dev.service" "podman-shiku-world-home-dev-db.service" ];
    script = ''
      ${pkgs.podman}/bin/podman network exists shiku-dev-net || \
      ${pkgs.podman}/bin/podman network create shiku-dev-net
      '';
  };
  secrets."docker-registry.password".file = ./secrets/docker-registry.password.age;
  secrets."htpasswd".file = ./secrets/dockerreg-htpasswd.age;
  secrets."shiku-world-home-dev-db-credentials".file = ./secrets/shiku-world-home-dev-db-credentials.age;
  virtualisation.oci-containers.containers = {
    "shiku-world-docker-registry" = {
      image = "registry:2";
      ports = ["5000:5000"];
      volumes = [
        "${registryDataPath}:/var/lib/registry"
      ];
      environment = {
        REGISTRY_HTTP_ADDR = "0.0.0.0:5000";
        REGISTRY_STORAGE_FILESYSTEM_ROOTDIRECTORY = "/var/lib/registry";
      };
    };
    "shiku-world-golem-bot" = {
      image = "build.shiku.world/golem-bot:latest";
      login = credentials;
      volumes = [
        "${golemPath}:/app/storage"
      ];
    };
    "shiku-world-medium-dev" = {
      image = "dreg.shiku.world/shiku-world-medium-dev:0.6.24";
      login = credentials;
      ports = ["8089:80"];
    };
    "shiku-world-home-dev" = {
      image = "dreg.shiku.world/shiku-world-home-dev:0.5.3";
      login = credentials;
      ports = ["9001:9001" "3030:3030"];
      dependsOn = [ "shiku-world-home-dev-db" ];
      autoStart = false;
      volumes = [
        "${shikuWorldHomeDevResourcePath}:/app/target/release/out"
      ];
      extraOptions = [ "--network=shiku-dev-net"];
    };
    "shiku-world-home-dev-db" = {
      image = "postgres:14.3";
      volumes = [
        "${shikuWorldHomeDbDevPath}:/var/lib/postgresql/data"
      ];
      extraOptions = [ "--network=shiku-dev-net" ];
      environmentFiles = ["/run/secrets/shiku-world-home-dev-db-credentials"];
    };
    "shiku-world-status" = {
      image = "build.shiku.world/shiku-world-status:latest";
      login = credentials;
      ports = ["3333:3000"];
    };
    "shiku-world-status-dev" = {
      image = "dreg.shiku.world/shiku-world-status-dev:0.1.4";
      login = credentials;
      ports = ["3334:3000"];
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
