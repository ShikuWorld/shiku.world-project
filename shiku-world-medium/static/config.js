let options;

$(function () {
  $("#shiku-conf").submit(function (e) {
    e.preventDefault();
    options = {};
    $("input[type=text]").each(function () {
      const self = $(this);
      options[self.attr("name")] = self.val();
    });
    Twitch.ext.configuration.set("broadcaster", "1", JSON.stringify(options));
  });
});

Twitch.ext.configuration.onChanged(function () {
  if (Twitch.ext.configuration.broadcaster) {
    try {
      const config = JSON.parse(Twitch.ext.configuration.broadcaster.content);

      if (typeof config === "object") {
        options = config;
        if (options.websocketurl) {
          $('input[name="websocketurl"]').val(options.websocketurl);
        }
        if (options.resourceUrl) {
          $('input[name="resourceUrl"]').val(options.resourceUrl);
        }
        if (options.twitchAuthRedirect) {
          $('input[name="twitchAuthRedirect"]').val(options.twitchAuthRedirect);
        }
        if (options.mainDoorStatusUrl) {
          $('input[name="mainDoorStatusUrl"]').val(options.mainDoorStatusUrl);
        }
        if (options.backDoorStatusUrl) {
          $('input[name="backDoorStatusUrl"]').val(options.backDoorStatusUrl);
        }
      } else {
        console.log("Invalid config");
      }
    } catch (e) {
      console.log("Invalid config");
    }
  }
});
