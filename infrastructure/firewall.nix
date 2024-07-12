{... }:
{
  networking.firewall = {
    interfaces."podman+" = {
      allowedUDPPorts = [ 53 ];
    };
  };
  networking.firewall.allowedTCPPorts = [ 23 80 443 ];
}
