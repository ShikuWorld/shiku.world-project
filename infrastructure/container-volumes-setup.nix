{ config, pkgs, ... }:

let
  folderToUpload = builtins.path {
    path = ./init-volume.d;
    name = "volumes";
  };
in
{
  system.activationScripts.init-c-volumes = {
    deps = [ "specialfs" ];
    text = ''
      echo '[init container volumes (mod)] Setting up'
      target="/var/lib/podman/volumes"
      mkdir -p "$target"
      cp -rn ${folderToUpload}/* "$target" || true
    '';
  };
}
