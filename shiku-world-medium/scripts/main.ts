import { watch_everything } from "./watch_everything";
import { build_code } from "./build_code";
import { set_env } from "./set_env";
import { copy_statics } from "./copy_statics";
import { hot_reload } from "./hot-reload";
import { getLogger } from "./log";
import { rmSync } from "fs";

const log = getLogger("");

async function main(command: string) {
  log.trace(`Executing ${command}`);
  switch (command) {
    case "watch-everything":
      await watch_everything();
      break;
    case "build":
      await build_code();
      break;
    case "clear-build":
      rmSync("./build", { recursive: true, force: true });
      break;
    case "copy-statics":
      await copy_statics();
      break;
    case "env":
      await set_env();
      break;
    case "hot-reload":
      await hot_reload();
      break;
    default:
      log.error(`Unknown command "${command}"`);
      break;
  }
}

const args = process.argv.slice(2);

if (!args[0]) {
  console.error("No argument given!");
}

main(args[0]).finally(() => {});
