import * as fs from "fs";
import * as dotenv from "dotenv";
import { getLogger } from "./log";
const targetPath = "./client/environment.ts";

const log = getLogger("");

export const set_env = () => {
  dotenv.config();

  // `environment.ts` file structure
  const envConfigFile = `export const environment = {
   wsSocketUrl: '${process.env.WS_SOCKET_URL}',
   resourceUrl: '${process.env.RESOURCE_URL}',
   twitchAuthRedirect: '${process.env.TWITCH_AUTH_REDIRECT}',
   mainDoorStatusUrl: '${process.env.MAIN_DOOR_STATUS_URL}',
   backDoorStatusUrl: '${process.env.BACK_DOOR_STATUS_URL}',
};
`;

  log.trace("Writing env file.");
  fs.writeFileSync(targetPath, envConfigFile);
};
