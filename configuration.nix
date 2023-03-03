{ pkgs, config, ... }: {
  imports = [
    ./hardware-configuration.nix
    ./secrets.nix
    ./webserver.nix
    ./firewall.nix
    ./helloworld.nix
  ];
  secrets.testConf.file = ./secrets/test.conf.age;
  virtualisation.podman.enable = true;
  system.stateVersion="23.05";
  boot.cleanTmpDir = true;
  zramSwap.enable = true;
  networking.hostName = "v2202302194870220598";
  networking.domain = "powersrv.de";
  services.openssh.enable = true;
  users.users.root.openssh.authorizedKeys.keys = [
    "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDVgJbW5w7XBpp6MY5o8rLyWGJkN4iWjXvY6k/yK0xaNiYIY3I5q+CAt6dZFGtNQzVufTCVP9UnkcpIL6xy/gkI5uhpVkkGnoZXAB6h5eSEB6nr57/8up/mOlPtaAnYu/uKolcSqDNcTN9yfHKj8ii9H8NiGFRQuaSGRnhNPKpXxxjKHHF/HXat3NPH+MEXqUHwLjwH4BiloirFM+yGYOz6xgFtIZJ1NaZJjmVQ0A3AeLKg61FRnHdqaoOpgBt5xr/YzI4efh0dYQB6ivTMNd1dxnrd3L2EA0e+sCwgjFtGBOMA92G1g9PEDvlZW9E/c4O8AAJpx8Z+hm26O3PWFFABZhm5QUbnEfo1ULkJgtStKoSzDj6892jEP9RMcRugXfDpaWnWrbfc1kXP30P1XSzZwEEZatOFlg7SFaBy8mdTsX2WzFYvN4UeDXyXL9I1tnC7GaCx7RmmEEaSx8Lc2Bt/bjDW1iuBmlHGzA0ge7P7q4+QTrE8ypx1VrMZcYRQfBpEhUh3lt022qtKrtNUjcymrR7iC8tYzh9fyCNolfq1RhEbTSXDPAUfYig53sME7SfI8egMuY4N4xSHJkdjwCPnReAsqJ6stIcqije08MsRUmfo8ycAqsvnGuWrEjMeWSEzQ/AIzzn9w5nk+emSPRnBkW82DnQ7kFCz9Qwa4H+IRQ== shikuworld@gmail.com"
  ];
}
