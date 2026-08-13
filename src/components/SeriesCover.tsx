import { convertFileSrc } from "@tauri-apps/api/core";
import { appDataDir, join } from "@tauri-apps/api/path";
import { useEffect, useState } from "react";
import type { CoverAsset } from "../domain/model";
import { TextCover } from "./TextCover";

export function SeriesCover({ cover, title, large = false }: { cover: CoverAsset | null; title: string; large?: boolean }) {
  const [source, setSource] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    setFailed(false);
    if (!cover) { setSource(null); return; }
    void (async () => setSource(convertFileSrc(await join(await appDataDir(), cover.relativePath))))();
  }, [cover]);
  if (!source || failed) return <TextCover title={title} large={large} />;
  return <img className={large ? "series-cover large" : "series-cover"} src={source} alt={`${title} 封面`} onError={() => setFailed(true)} />;
}
