{ pkgs, ... }:
{
  security.acme.acceptTerms = true;
  security.acme.certs = {
    "status.shiku.world".email = "server@shiku.world";
    "files.shiku.world".email = "server@shiku.world";
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
      "files.shiku.world" = {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:8088";
          extraConfig = ''
             proxy_buffers 8 32k;
             proxy_buffer_size 64k;
             client_max_body_size 75M;
             proxy_headers_hash_max_size 512;
             proxy_headers_hash_bucket_size 128;
          '';
        };
      };
    };
  };
}
