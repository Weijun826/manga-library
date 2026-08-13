import { open } from "@tauri-apps/plugin-dialog";

export async function selectCoverImage(): Promise<string | null> {
  return open({
    multiple: false,
    directory: false,
    filters: [{ name: "漫畫封面", extensions: ["jpg", "jpeg", "png", "webp"] }],
  });
}
