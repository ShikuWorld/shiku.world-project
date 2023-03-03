{ pkgs, ... }:
{
  virtualisation.oci-containers.containers = {
    "shiku-world-golem-bot" = {
      image = "build.shiku.world/golem-bot:latest";
    };
  };
}
