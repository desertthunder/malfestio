import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SearchInput } from "../SearchInput";

const navigateMock = vi.fn();
vi.mock("@solidjs/router", () => ({ useNavigate: () => navigateMock }));

describe("SearchInput", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("renders correctly", () => {
    render(() => <SearchInput />);
    expect(screen.getByPlaceholderText("Search decks, cards...")).toBeInTheDocument();
  });

  it("updates query on input", () => {
    render(() => <SearchInput />);
    const input = screen.getByPlaceholderText("Search decks, cards...") as HTMLInputElement;
    fireEvent.input(input, { target: { value: "test query" } });
    expect(input.value).toBe("test query");
  });

  it("navigates on submit with query", () => {
    render(() => <SearchInput />);
    const input = screen.getByPlaceholderText("Search decks, cards...");
    fireEvent.input(input, { target: { value: "test" } });
    fireEvent.submit(input.closest("form")!);
    expect(navigateMock).toHaveBeenCalledWith("/search?q=test");
  });

  it("does not navigate on empty submit", () => {
    render(() => <SearchInput />);
    const input = screen.getByPlaceholderText("Search decks, cards...");
    fireEvent.submit(input.closest("form")!);
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it("initializes with initialQuery prop", () => {
    render(() => <SearchInput initialQuery="initial" />);
    const input = screen.getByPlaceholderText("Search decks, cards...") as HTMLInputElement;
    expect(input.value).toBe("initial");
  });
});
