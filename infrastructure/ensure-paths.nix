{ lib, config, pkgs, ... }:

let
  createVolumeDirectory = volPath:
    ''
      mkdir -p ${volPath}
      echo "Created volume directory ${volPath}"
    '';
in {
  options = {
    ensurePaths = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = ["/etc/test"];
      description = ''
        Array of volumes to be created.
      '';
    };
  };

  config = lib.mkIf (lib.hasAttr "ensurePaths" config) {
    system.activationScripts.createVolumeDirectories = {
     text = ''
       ${builtins.concatStringsSep "\n" (map createVolumeDirectory config.ensurePaths)}
     '';
    };
  };
}
