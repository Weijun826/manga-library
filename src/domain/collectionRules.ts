import type { AvailabilityStatus, CollectionFlags } from "./model";

export interface VolumeWithCollection {
  availabilityStatus: AvailabilityStatus;
  collection: CollectionFlags;
}

export function applyCollectionPatch(
  current: CollectionFlags,
  patch: Partial<CollectionFlags>,
): CollectionFlags {
  const next = { ...current, ...patch };
  if (next.isOwned) next.isWishlisted = false;
  return next;
}

export function isMissingVolume(volume: VolumeWithCollection): boolean {
  return volume.availabilityStatus === "released" && !volume.collection.isOwned;
}

export function summarizeEdition(volumes: VolumeWithCollection[]) {
  return {
    known: volumes.filter((volume) => volume.availabilityStatus !== "unknown").length,
    released: volumes.filter((volume) => volume.availabilityStatus === "released").length,
    owned: volumes.filter((volume) => volume.collection.isOwned).length,
    read: volumes.filter((volume) => volume.collection.isRead).length,
    wishlisted: volumes.filter((volume) => volume.collection.isWishlisted).length,
    missing: volumes.filter(isMissingVolume).length,
  };
}
