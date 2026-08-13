import { describe, expect, it } from "vitest";

import {
  upcomingUnownedFixture,
  tenVolumeFixture,
  unknownUnownedFixture,
} from "../test/fixtures/library";
import {
  applyCollectionPatch,
  isMissingVolume,
  summarizeEdition,
} from "./collectionRules";

describe("collection state rules", () => {
  it("owning a wished-for volume removes it from the wishlist", () => {
    expect(
      applyCollectionPatch(
        { isOwned: false, isRead: false, isWishlisted: true },
        { isOwned: true },
      ),
    ).toEqual({ isOwned: true, isRead: false, isWishlisted: false });
  });

  it("removing ownership preserves read state", () => {
    expect(
      applyCollectionPatch(
        { isOwned: true, isRead: true, isWishlisted: false },
        { isOwned: false },
      ).isRead,
    ).toBe(true);
  });

  it("counts only released unowned volumes as missing", () => {
    expect(summarizeEdition(tenVolumeFixture).missing).toBe(4);
    expect(isMissingVolume(upcomingUnownedFixture)).toBe(false);
    expect(isMissingVolume(unknownUnownedFixture)).toBe(false);
  });

  it("summarizes each collection state without treating read volumes as owned", () => {
    expect(summarizeEdition(tenVolumeFixture)).toEqual({
      known: 9,
      released: 8,
      owned: 4,
      read: 2,
      wishlisted: 2,
      missing: 4,
    });
  });
});
