import type { ReviewCard } from "$lib/model";
import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { StudySession } from "../StudySession";

vi.mock(
  "$lib/api",
  () => ({ api: { submitReview: vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve({}) }) } }),
);

const mockCards: ReviewCard[] = [{
  review_id: "review-1",
  card_id: "card-1",
  deck_id: "deck-1",
  deck_title: "Test Deck",
  front: "What is 2+2?",
  back: "4",
  media_url: undefined,
  hints: [],
  due_at: new Date().toISOString(),
}, {
  review_id: "review-2",
  card_id: "card-2",
  deck_id: "deck-1",
  deck_title: "Test Deck",
  front: "What is the capital of France?",
  back: "Paris",
  media_url: undefined,
  hints: ["It's a famous city"],
  due_at: new Date().toISOString(),
}];

describe("StudySession", () => {
  afterEach(cleanup);

  it("renders card front initially", () => {
    const onComplete = vi.fn();
    const onExit = vi.fn();

    render(() => <StudySession cards={mockCards} onComplete={onComplete} onExit={onExit} />);

    expect(screen.getByText("What is 2+2?")).toBeInTheDocument();
    expect(screen.getByText("Press Space or click to reveal")).toBeInTheDocument();
  });

  it("shows progress indicator", () => {
    const onComplete = vi.fn();
    const onExit = vi.fn();

    render(() => <StudySession cards={mockCards} onComplete={onComplete} onExit={onExit} />);

    expect(screen.getByText(/Card 1 of 2/i)).toBeInTheDocument();
  });

  it("flips card on click", async () => {
    const onComplete = vi.fn();
    const onExit = vi.fn();

    render(() => <StudySession cards={mockCards} onComplete={onComplete} onExit={onExit} />);

    expect(screen.getByText("What is 2+2?")).toBeInTheDocument();

    const cardElement = screen.getByText("What is 2+2?").closest("div[class*='cursor-pointer']");
    if (cardElement) {
      fireEvent.click(cardElement);
    }

    expect(await screen.findByText("How well did you know this?")).toBeInTheDocument();
  });

  it("flips back to front on second click", async () => {
    const onComplete = vi.fn();
    const onExit = vi.fn();

    render(() => <StudySession cards={mockCards} onComplete={onComplete} onExit={onExit} />);

    const cardElement = screen.getByText("What is 2+2?").closest("div[class*='cursor-pointer']");
    if (cardElement) fireEvent.click(cardElement);

    expect(await screen.findByText("How well did you know this?")).toBeInTheDocument();

    if (cardElement) fireEvent.click(cardElement);
    expect(await screen.findByText("Press Space or click to reveal")).toBeInTheDocument();
    expect(screen.queryByText("How well did you know this?")).not.toBeInTheDocument();
  });

  it("shows keyboard hints conditionally", async () => {
    const onComplete = vi.fn();
    const onExit = vi.fn();

    render(() => <StudySession cards={mockCards} onComplete={onComplete} onExit={onExit} />);

    expect(screen.getByText("Space: Flip")).toBeInTheDocument();
    expect(screen.getByText("Esc: Exit")).toBeInTheDocument();

    // Initially hidden
    expect(screen.queryByText("1-6: Grade")).not.toBeInTheDocument();
    expect(screen.queryByText("E: Edit")).not.toBeInTheDocument();

    // Flip card
    const cardElement = screen.getByText("What is 2+2?").closest("div[class*='cursor-pointer']");
    if (cardElement) fireEvent.click(cardElement);

    // Now visible
    expect(await screen.findByText("1-6: Grade")).toBeInTheDocument();
    expect(screen.getByText("E: Edit")).toBeInTheDocument();
  });

  it("calls onExit when exit button is clicked", () => {
    const onComplete = vi.fn();
    const onExit = vi.fn();

    render(() => <StudySession cards={mockCards} onComplete={onComplete} onExit={onExit} />);

    const exitButton = screen.getByRole("button", { name: /✕ Exit/i });
    fireEvent.click(exitButton);

    expect(onExit).toHaveBeenCalled();
  });
});
