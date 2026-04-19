import { invoke as realInvoke } from "@tauri-apps/api/core";
const inTauri = "__TAURI_INTERNALS__" in window;

export const invoke: typeof realInvoke = inTauri
  ? realInvoke
  : async (cmd, args) => {
      const { mockInvoke } = await import("./mocks/invoke");
      return mockInvoke(cmd, args) as any;
    };
