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
    "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCTCTbtdzV1e6PcAs3/S4sU+Z5J3cilVhB4bkuUM5NaoQqoYvWcr6LH31hv8Mz5MJw6Wqb9vHFPC4uBmaS+24TbGez+8Xpoajfu5IxOb2QitC/ym5KD1TMXM/kKEQmX+zpRjBpNM8SOVveoRxZg863+/QIU3rK5s5hsd9BXROYaPTB0iaJoOZwZZDhKjPaqlnjn7cpYYAOAPl6ujm1lPPlRfWVXbw/7bs2ZzuE+sfaWmrBgzoZRAidx2SGB7XVp2H1ixH9F7XgICkju1qfFYzAAcBo50zbRFvxN2D/UTX67VFqZDCDCX4UAxVY+8SwtVT0GdFKVx+RAIn6QVsX2rkjm3X2wv0H4RlojR9Q8BhqQmj6YNwiQfHtjr4dnIdOjh2TGYm7kYldEfOJUoGGhxsqXfqxhG4d7mz+6T0jEJ4bFslJZdRbRLhhUXf2RrilmzSasnqR2bFUarE5syaZTvA3+fOkq3DMvOivQ00vZXPJkVqEnzvS7Q1+0VU5D3GFhKTM= shiku@nixos"
  ];
}
