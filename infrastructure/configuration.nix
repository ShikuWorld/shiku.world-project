{ pkgs, config, ... }: {
  environment.systemPackages = with pkgs; [
    nodejs
  ];
  imports = [
    ./hardware-configuration.nix
    ./secrets.nix
    ./firewall.nix
    ./container-control-service.nix
    ./container-volumes-setup.nix
    ./containers.nix
    ./webserver.nix
  ];
  virtualisation.podman.enable = true;
  system.stateVersion="23.05";
  boot.tmp.cleanOnBoot = true;
  zramSwap.enable = true;
  networking.hostName = "v2202302194870220598";
  networking.domain = "powersrv.de";
  services.openssh.enable = true;
  users.users.root.openssh.authorizedKeys.keys = [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOX8sysxyGjBvIzZzY5+YIoUO4Ot8lZ/K/Zsrfdj8N1W root@Kuroneko"
  ];
}
