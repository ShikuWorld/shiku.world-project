export const use_medium_api = (): typeof window.medium =>
  window.medium
    ? window.medium
    : {
        twitch_login: () => Promise.resolve(),
        communication_state: {} as any,
      };
