export const use_medium_api = (): typeof window.medium =>
  window.medium
    ? window.medium
    : {
        twitch_login: () => Promise.resolve(),
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        communication_state: {} as any,
        swap_main_render_instance: () => {},
      };
