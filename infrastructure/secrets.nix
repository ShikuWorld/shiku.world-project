# Adapted from https://github.com/ryantm/agenix
# Modified for personal use
# Copied afterwards into this project.
{ pkgs, lib, config, ...}:
let
  rootPath = "/run/secrets";
  keyPath = "/etc/nixos/root.key";

  installSecretBase = {cfg, secretType}: ''
    echo "decrypting ${secretType.file} to ${secretType.path}..."
    TMP_FILE="${secretType.path}.tmp"
    mkdir -p $(dirname ${secretType.path})
    (
      umask u=r,g=,o=
      ${cfg.envVars} ${pkgs.age}/bin/age --decrypt -i ${cfg.key} -o "$TMP_FILE" "${secretType.file}"
    )
    ${cfg.postCommands}
    mv -f "$TMP_FILE" '${secretType.path}'
  '';

  installSecret = secretType: installSecretBase { inherit secretType; cfg = {
    key = keyPath;

    envVars = "LANG=${config.i18n.defaultLocale}";
    postCommands = ''
      chmod 777 "$TMP_FILE"
      chown ${secretType.owner}:${secretType.group} "$TMP_FILE"
    '';
  }; };

  users = config.users.users;

  isRootSecret = st: (st.owner == "root" || st.owner == "0") && (st.group == "root" || st.group == "0");
  isNotRootSecret = st: !(isRootSecret st);

  rootOwnedSecrets = builtins.filter isRootSecret (builtins.attrValues config.secrets);
  installRootOwnedSecrets = builtins.concatStringsSep "\n" ([ "echo '[agenix (mod)] decrypting root secrets...'" ] ++ (map installSecret rootOwnedSecrets));

  nonRootSecrets = builtins.filter isNotRootSecret (builtins.attrValues config.secrets);
  installNonRootSecrets = builtins.concatStringsSep "\n" ([ "echo '[agenix (mod)] decrypting non-root secrets...'" ] ++ (map installSecret nonRootSecrets));
in
{
  options = (
    with lib;
    with lib.types;
    let
      secretType = types.submodule ({ config, ... }: {
        options = {
          name = mkOption {
            type = types.str;
            default = config._module.args.name;
            description = ''
              Name of the file used in /run/secrets
            '';
          };
          file = mkOption {
            type = types.path;
            description = ''
              Age file the secret is loaded from.
            '';
          };
          path = mkOption {
            type = types.str;
            default = "${rootPath}/${config.name}";
            description = ''
              Path where the decrypted secret is installed.
            '';
          };
          mode = mkOption {
            type = types.str;
            default = "0400";
            description = ''
              Permissions mode of the in octal.
            '';
          };
          owner = mkOption {
            type = types.str;
            default = "0";
            description = ''
              User of the file.
            '';
          };
          group = mkOption {
            type = types.str;
            default = users.${config.owner}.group or "0";
            description = ''
              Group of the file.
            '';
          };
        };
      });
    in
    {
      secrets = mkOption {
        type = types.attrsOf secretType;
        default = { };
        description = ''
          Attrset of secrets.
        '';
      };
    }
  );

  config = lib.mkIf ((builtins.length (builtins.attrNames config.secrets)) > 0) {
    system.activationScripts.secretDecryptionKey = {
      deps = [ "specialfs" ];
      text = ''
        echo '[agenix (mod)] Setting up secrets dir'

        ##
        # Create /run/secrets if it doesn't exist yet.
        if [ ! -d ${rootPath} ]; then
          mkdir ${rootPath}
        fi
      '';
    };

    system.activationScripts.rootSecrets = {
      deps = [ "secretDecryptionKey" ];
      text = installRootOwnedSecrets;
    };

    system.activationScripts.nonRootSecrets = {
      deps = [ "secretDecryptionKey" "users" "groups" ];
      text = installNonRootSecrets;
    };

    system.activationScripts.secretsBoundary = {
      deps = [ "rootSecrets" "nonRootSecrets" ];
      text = "";
    };

    system.activationScripts.users.deps = [ "rootSecrets" ];
    system.activationScripts.groups.deps = [ "rootSecrets" ];
  };
}
