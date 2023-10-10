import * as Logger from "js-logger";
import {ILogger} from "js-logger";

Logger.setDefaults({
  defaultLevel: (Logger as unknown as ILogger).TRACE,
});
Logger.setHandler((messages, context) => {
  if (context.level.value > (Logger as unknown as ILogger).WARN.value) {
    for (const message of messages) {
      process.stderr.write(message, 'utf8');
    }
    return;
  }
  for (const message of messages) {
    process.stdout.write(message, 'utf8');
  }
});

export const getLogger = (name: string): ILogger => {
  return Logger.get(name);
};
