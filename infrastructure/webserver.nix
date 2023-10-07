{ pkgs, ... }:
{
  security.acme.acceptTerms = true;
  security.acme.certs = {
    "status.shiku.world".email = "server@shiku.world";
  };
  services.nginx = {
    package = pkgs.nginxMainline;
    enable = true;
    logError = "stderr debug";
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    virtualHosts = {
      "status.shiku.world" = {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:3333";
        };
      };
    };
  };
}
