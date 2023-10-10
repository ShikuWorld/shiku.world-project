import * as child_process from "child_process";
import { watch } from "chokidar";
import { copy_statics } from "./copy_statics";
import { build_code } from "./build_code";
import { getLogger } from "./log";
import { hot_reload } from "./hot-reload";

const log = getLogger("");

export const watch_everything = async () => {
  const codeWatcher = watch(["gui/src/**/*", "client/**/*"]);
  log.trace("Watching files...");
  await build_code();
  codeWatcher.on("change", async () => {
    await build_code();
  });
  codeWatcher.on("error", () => {
    console.error("wat");
  });

  const staticWatcher = watch(["static/**/*"]);
  log.trace("Watching static files...");
  copy_statics();
  staticWatcher.on("change", () => {
    copy_statics();
  });

  const envWatcher = watch([".env"]);
  log.trace("Watching env file.");
  child_process.spawnSync("yarn.cmd", ["env"], { env: {} });
  envWatcher.on("change", async () => {
    const output = await child_process.spawnSync("yarn.cmd", ["env"], {
      env: {},
    });
    log.trace(output.stdout.toString());
  });

  hot_reload();
};
