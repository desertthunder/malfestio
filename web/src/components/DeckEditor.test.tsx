import { api } from "$lib/api";
import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { DeckEditor } from "./DeckEditor";

vi.mock(
  "$lib/api",
  () => ({ api: { post: vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve({}) }) } }),
);

describe("DeckEditor", () => {
  afterEach(cleanup);

  it("renders form fields", () => {
    render(() => <DeckEditor />);
    expect(screen.getByLabelText(/Title/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Description/i)).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: /Visibility/i })).toBeInTheDocument();
  });

  it("shows shared with input when SharedWith is selected", () => {
    render(() => <DeckEditor />);
    const select = screen.getByRole("combobox", { name: /Visibility/i });

    fireEvent.change(select, { target: { value: "SharedWith" } });

    expect(screen.getByPlaceholderText(/did:plc/i)).toBeInTheDocument();
  });

  it("submits correct payload for Private deck", async () => {
    render(() => <DeckEditor />);

    fireEvent.input(screen.getByLabelText(/Title/i), { target: { value: "My Deck" } });
    fireEvent.change(screen.getByRole("combobox", { name: /Visibility/i }), { target: { value: "Private" } });

    const submitBtn = screen.getByRole("button", { name: /Create Deck/i });
    fireEvent.click(submitBtn);

    expect(api.post).toHaveBeenCalledWith(
      "/decks",
      expect.objectContaining({ title: "My Deck", visibility: { type: "Private" } }),
    );
  });

  it("submits correct payload for SharedWith deck", async () => {
    render(() => <DeckEditor />);

    fireEvent.input(screen.getByLabelText(/Title/i), { target: { value: "Shared Deck" } });

    const select = screen.getByRole("combobox", { name: /Visibility/i });
    fireEvent.change(select, { target: { value: "SharedWith" } });

    const sharedInput = screen.getByPlaceholderText(/did:plc/i);
    fireEvent.input(sharedInput, { target: { value: "did:plc:123, did:plc:456" } });

    const submitBtn = screen.getByRole("button", { name: /Create Deck/i });
    fireEvent.click(submitBtn);

    expect(api.post).toHaveBeenCalledWith(
      "/decks",
      expect.objectContaining({
        title: "Shared Deck",
        visibility: { type: "SharedWith", content: ["did:plc:123", "did:plc:456"] },
      }),
    );
  });
});
