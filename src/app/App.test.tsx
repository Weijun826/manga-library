import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("renders the Manga Library product identity", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "漫畫書庫" })).toBeVisible();
  });
});
