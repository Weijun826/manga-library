export function TextCover({ title, large = false }: { title: string; large?: boolean }) {
  const initials = title.trim().slice(0, 2) || "冊";
  return <span className={large ? "text-cover large" : "text-cover"} role="img" aria-label={`${title} 的文字封面`}>{initials}</span>;
}
