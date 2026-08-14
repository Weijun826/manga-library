import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { VolumeWithCollection } from "../domain/model";
import { VolumeEditorDrawer } from "./VolumeEditorDrawer";

afterEach(cleanup);

const volume: VolumeWithCollection = {
  id: "volume-1", editionId: "edition-1", displayLabel: "1", sortKey: "0:000001.000",
  titleOverride: null, isbn10: null, isbn13: "9789572690017", translator: null,
  availabilityStatus: "released", releaseDate: null, releaseDatePrecision: "unknown",
  listPriceAmount: null, listPriceCurrency: null, cover: null,
  collection: { isOwned: false, isRead: true, isWishlisted: true, purchasePriceAmount: 120,
    purchasePriceCurrency: "TWD", acquiredOn: "2026-08-14", condition: "good",
    storageLocation: "書櫃 A", notes: "首刷" },
};

describe("VolumeEditorDrawer", () => {
  it("prefills every field and submits normalized form data", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn().mockResolvedValue(undefined);
    render(<VolumeEditorDrawer volume={volume} busy={false} onClose={vi.fn()} onSave={onSave} />);

    expect(screen.getByRole("dialog", { name: "編輯第 1 冊" })).toBeVisible();
    expect(screen.getByLabelText("ISBN")).toHaveValue("9789572690017");
    expect(screen.getByLabelText("入手日期")).toHaveValue("2026-08-14");
    expect(screen.getByLabelText("購買價格（TWD）")).toHaveValue(120);
    await user.clear(screen.getByLabelText("卷數／標籤"));
    await user.type(screen.getByLabelText("卷數／標籤"), " 1.5 ");
    await user.click(screen.getByLabelText("擁有"));
    expect(screen.getByLabelText("願望")).not.toBeChecked();
    expect(screen.getByLabelText("願望")).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "儲存變更" }));

    expect(onSave).toHaveBeenCalledWith(expect.objectContaining({
      displayLabel: "1.5",
      isbn: "9789572690017",
      collection: expect.objectContaining({ isOwned: true, isRead: true, isWishlisted: false }),
    }));
  });

  it("rejects invalid dates and negative prices without saving", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    render(<VolumeEditorDrawer volume={volume} busy={false} onClose={vi.fn()} onSave={onSave} />);
    await user.clear(screen.getByLabelText("入手日期"));
    await user.type(screen.getByLabelText("入手日期"), "2023-02-29");
    await user.clear(screen.getByLabelText("購買價格（TWD）"));
    await user.type(screen.getByLabelText("購買價格（TWD）"), "-1");
    await user.click(screen.getByRole("button", { name: "儲存變更" }));
    expect(screen.getByText("請輸入有效日期，例如 2026-08-14。")).toBeVisible();
    expect(screen.getByText("價格必須是 0 以上的整數。")).toBeVisible();
    expect(onSave).not.toHaveBeenCalled();
  });

  it("asks before closing dirty input", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    render(<VolumeEditorDrawer volume={volume} busy={false} onClose={onClose} onSave={vi.fn()} />);
    await user.type(screen.getByLabelText("備註"), "追加");
    await user.click(screen.getByRole("button", { name: "取消" }));
    expect(screen.getByRole("alertdialog", { name: "放棄未儲存變更？" })).toBeVisible();
    await user.click(screen.getByRole("button", { name: "繼續編輯" }));
    expect(onClose).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "取消" }));
    await user.click(screen.getByRole("button", { name: "放棄變更" }));
    expect(onClose).toHaveBeenCalledOnce();
  });
});
