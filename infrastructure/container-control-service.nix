{ config, pkgs, ... }:

let
  containerControlService = ./container-control-service/dist;
in
{
  systemd.services.containerControlService = {
    description = "Container Control Service";
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    restartIfChanged = true;
    serviceConfig = {
      Environment = "PATH=${pkgs.lib.makeBinPath [ pkgs.podman pkgs.systemd ]} HOME=/tmp/containerControlService/home XDG_RUNTIME_DIR=/tmp/containerControlService/xdg";
      ExecStart = "${pkgs.nodejs}/bin/node ${containerControlService}/index.js";
      ExecStartPre = "${pkgs.coreutils}/bin/mkdir -p /tmp/containerControlService/home /tmp/containerControlService/xdg";
      WorkingDirectory = "${containerControlService}";
      Restart = "always";
    };
  };
}
