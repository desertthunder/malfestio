import { api } from "$lib/api";
import Settings from "$pages/Settings";
import { cleanup, fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, type Mock, vi } from "vitest";

vi.mock("$lib/api", () => ({ api: { exportData: vi.fn() } }));

vi.stubGlobal("URL", { createObjectURL: vi.fn(() => "blob:test"), revokeObjectURL: vi.fn() });

describe("Settings page", () => {
  afterEach(cleanup);

  it("renders export buttons", () => {
    render(() => <Settings />);
    expect(screen.getByRole("heading", { name: "Export Data" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Export Decks" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Export Notes" })).toBeInTheDocument();
  });

  it("handles deck export", async () => {
    (api.exportData as Mock).mockResolvedValue({
      ok: true,
      blob: async () => new Blob(["test"], { type: "application/json" }),
    });

    render(() => <Settings />);

    const exportBtn = screen.getByRole("button", { name: "Export Decks" });
    fireEvent.click(exportBtn);

    expect(screen.getByText("Exporting...")).toBeInTheDocument();
    await waitFor(() => expect(api.exportData).toHaveBeenCalledWith("decks"));
  });

  it("handles notes export", async () => {
    (api.exportData as Mock).mockResolvedValue({
      ok: true,
      blob: async () => new Blob(["test"], { type: "application/json" }),
    });

    render(() => <Settings />);

    const exportBtn = screen.getByRole("button", { name: "Export Notes" });
    fireEvent.click(exportBtn);

    expect(screen.getByText("Exporting...")).toBeInTheDocument();
    await waitFor(() => expect(api.exportData).toHaveBeenCalledWith("notes"));
  });
});
