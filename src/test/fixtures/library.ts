import type { VolumeWithCollection } from "../../domain/collectionRules";

export const tenVolumeFixture: VolumeWithCollection[] = [
  { availabilityStatus: "released", collection: { isOwned: true, isRead: true, isWishlisted: false } },
  { availabilityStatus: "released", collection: { isOwned: true, isRead: false, isWishlisted: false } },
  { availabilityStatus: "released", collection: { isOwned: false, isRead: false, isWishlisted: true } },
  { availabilityStatus: "released", collection: { isOwned: false, isRead: false, isWishlisted: false } },
  { availabilityStatus: "released", collection: { isOwned: false, isRead: true, isWishlisted: false } },
  { availabilityStatus: "released", collection: { isOwned: false, isRead: false, isWishlisted: false } },
  { availabilityStatus: "released", collection: { isOwned: true, isRead: false, isWishlisted: false } },
  { availabilityStatus: "upcoming", collection: { isOwned: false, isRead: false, isWishlisted: true } },
  { availabilityStatus: "unknown", collection: { isOwned: false, isRead: false, isWishlisted: false } },
  { availabilityStatus: "released", collection: { isOwned: true, isRead: false, isWishlisted: false } },
];

export const upcomingUnownedFixture: VolumeWithCollection = {
  availabilityStatus: "upcoming",
  collection: { isOwned: false, isRead: false, isWishlisted: false },
};

export const unknownUnownedFixture: VolumeWithCollection = {
  availabilityStatus: "unknown",
  collection: { isOwned: false, isRead: false, isWishlisted: false },
};
