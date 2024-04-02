import { BitmapText } from "pixi.js";

import { RenderTypeTimer } from "./communication/api/bindings/RenderTypeTimer";

export function create_countdown(config: RenderTypeTimer): BitmapText {
  const text = new BitmapText({
    text: config.date,
    style: { fontFamily: config.font_family },
  });

  setInterval(set_new_time_to_text(config, text), 500);
  set_new_time_to_text(config, text)();

  return text;
}

function set_new_time_to_text(config: RenderTypeTimer, text: BitmapText) {
  return () => {
    const date = new Date(config.date);
    const diff = date.getTime() - Date.now();

    if (diff > 0) {
      text.text = format_countdown(diff);
      text.position.x = Math.round(-text.width / 2);
    } else {
      text.text = "HideThePainHarold";
    }
  };
}

function format_countdown(unix_timestamp: number): string {
  const unix_timestamp_seconds = unix_timestamp / 1000;

  const days = Math.floor(unix_timestamp_seconds / (60 * 60 * 24));
  const hours = Math.floor(
    (unix_timestamp_seconds - days * 60 * 60 * 24) / (60 * 60),
  );
  const minutes = Math.floor(
    (unix_timestamp_seconds - days * 60 * 60 * 24 - hours * 60 * 60) / 60,
  );
  const seconds = Math.floor(
    unix_timestamp_seconds -
      days * 60 * 60 * 24 -
      hours * 60 * 60 -
      minutes * 60,
  );

  return `${prefix_0(days)}:${prefix_0(hours)}:${prefix_0(minutes)}:${prefix_0(
    seconds,
  )}`;
}

function prefix_0(n: number): string {
  return n < 10 ? `0${n}` : `${n}`;
}
