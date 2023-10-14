import { getLogger } from "./log";
import * as browserSync from "browser-sync";

const log = getLogger("");
export const hot_reload = () => {
  log.debug("Starting hot reload");
  browserSync(
    {
      watch: true,
      files: "build/*.js",
      server: {
        baseDir: "build",
        index: "index.html",
      },
      https: true,
      port: 8080,
    },
    (err, _bs) => {
      console.log(err);
    },
  );
};
