import * as fs from "fs-extra";

// Get index html and replace stuff
export const copy_statics = async () => {
  const gui_includes = fs
    .readFileSync("./gui/dist/index.html")
    .toString()
    .split("\n")
    .filter((l) => /<script type="module" | <link rel="stylesheet"/g.test(l));

  fs.copySync("./gui/dist", "./build/", {
    overwrite: true,
    filter: (f) => !f.includes("index.html"),
  });

  const index = fs
    .readFileSync("./static/index.html", "utf-8")
    .replace(/<!--/g, "")
    .replace(/-->/g, "")
    .replace(/{{timestamp}}/g, Date.now().toString())
    .replace(/{{twitch}}/, "")
    .replace(/{{bodyClass}}/, "")
    .replace(/{{gui}}/, gui_includes.join("\n    "))
    .replace(
      /{{input_methods}}/,
      `<script src="keyboard-input.js?t=${Date.now().toString()}"></script>
    <script src="controller-input.js?t=${Date.now().toString()}"></script>
    <script src="mouse-input.js?t=${Date.now().toString()}"></script>`,
    )
    .replace(
      /{{chat}}/,
      `<div id="twitch-chat" class="twitch-chat-hidden">
        <div id="toggle-chat">»</div>
        <iframe src="https://www.twitch.tv/embed/shikusworld/chat?parent=localhost&parent=shiku.world&parent=dev.shiku.world&darkpopout"
                height="100%"
                width="100%">
        </iframe>
    </div>`,
    );

  fs.writeFileSync("./build/index.html", index);

  const video_overlay = fs
    .readFileSync("./static/index.html", "utf-8")
    .replace(/<!--/g, "")
    .replace(/-->/g, "")
    .replace(/{{timestamp}}/g, Date.now().toString())
    .replace(
      /{{twitch}}/,
      '<script src="https://extension-files.twitch.tv/helper/v1/twitch-ext.min.js"></script>',
    )
    .replace(/{{chat}}/, `<div id="door" class="door--locked"></div>`)
    .replace(/{{bodyClass}}/, "twitch-extension-body")
    .replace(/{{gui}}/, gui_includes.join("\n"))
    .replace(
      /{{input_methods}}/,
      `<script src="controller-input.js?t=${Date.now().toString()}"></script>
    <script src="mouse-input.js?t=${Date.now().toString()}"></script>`,
    );

  fs.writeFileSync("./build/video_overlay.html", video_overlay);

  fs.copyFileSync("./static/config.js", "./build/config.js");
  fs.copyFileSync(
    "./static/jquery-3.6.0.min.js",
    "./build/jquery-3.6.0.min.js",
  );
  fs.copyFileSync("./static/config.css", "./build/config.css");
  fs.copyFileSync("./static/config.html", "./build/config.html");

  fs.copyFileSync("./static/style.css", "./build/style.css");

  fs.copySync("./static/fonts", "./build/fonts");
  fs.copyFileSync("./static/door_closed.png", "./build/door_closed.png");
  fs.copyFileSync(
    "./static/door_locked_greyish.png",
    "./build/door_locked_greyish.png",
  );
  fs.copyFileSync("./static/door_open.png", "./build/door_open.png");
};
