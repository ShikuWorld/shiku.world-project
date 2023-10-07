{ pkgs, ... }:
{
  security.acme.acceptTerms = true;
  security.acme.certs = {
    "nixtest.shiku.world".email = "server@shiku.world";
    "dockertest.shiku.world".email = "server@shiku.world";
  };
  services.nginx.package = pkgs.nginxMainline;
  services.nginx.enable = true;
  services.nginx.logError = "stderr debug";
  services.nginx.virtualHosts = {
    "dockertest.shiku.world" = {
      forceSSL = true;
      enableACME = true;
      locations."/" = {
        proxyPass = "http://localhost:3333";
      };
    };
  };
}
