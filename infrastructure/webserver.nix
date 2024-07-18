{ pkgs, ... }:
{
  security.acme.acceptTerms = true;
  security.acme.certs = {
    "status.shiku.world".email = "server@shiku.world";
    "dev-status.shiku.world".email = "server@shiku.world";
    "files.shiku.world".email = "server@shiku.world";
    "resources.shiku.world".email = "server@shiku.world";
    "dev-home-status.shiku.world".email = "server@shiku.world";
    "dev-home.shiku.world".email = "server@shiku.world";
    "dev.shiku.world".email = "server@shiku.world";
    "dreg.shiku.world".email = "server@shiku.world";
  };
  services.nginx = {
    package = pkgs.nginxMainline;
    enable = true;
    logError = "stderr debug";
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    virtualHosts = {
      "dreg.shiku.world" = {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:5000";
        };
        basicAuthFile = "/run/secrets/htpasswd";
        extraConfig = ''
           client_max_body_size 0;
        '';
      };
      "status.shiku.world" = {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:3333";
        };
      };
      "resources.shiku.world" = {
        enableACME = true;
        forceSSL = true;
        locations."/static" = {
          proxyPass = "http://127.0.0.1:8083";
        };
        locations."/ws" = {
          proxyPass = "http://127.0.0.1:8083";
          extraConfig = ''
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "Upgrade";
            proxy_set_header Host $host;
          '';
        };
      };
      "dev-status.shiku.world" = {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:3334";
        };
      };
      "dev.shiku.world" = {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:8089";
        };
      };
      "dev-home-status.shiku.world" = {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:3030";
        };
      };
      "dev-home.shiku.world" = {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:9001";
          extraConfig = ''
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "Upgrade";
            proxy_set_header Host $host;
         '';
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
