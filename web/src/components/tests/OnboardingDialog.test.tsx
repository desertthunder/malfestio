import { cleanup, fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OnboardingDialog } from "../OnboardingDialog";

vi.mock("$lib/api", () => ({ api: { updatePreferences: vi.fn() } }));

describe("OnboardingDialog", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("renders when open", () => {
    render(() => <OnboardingDialog open={true} onComplete={() => {}} />);
    expect(screen.getByText("Welcome to Malfestio")).toBeInTheDocument();
  });

  it("does not render when closed", () => {
    render(() => <OnboardingDialog open={false} onComplete={() => {}} />);
    expect(screen.queryByText("Welcome to Malfestio")).not.toBeInTheDocument();
  });

  it("displays all three persona options", () => {
    render(() => <OnboardingDialog open={true} onComplete={() => {}} />);
    expect(screen.getByText("Learner")).toBeInTheDocument();
    expect(screen.getByText("Creator")).toBeInTheDocument();
    expect(screen.getByText("Curator")).toBeInTheDocument();
  });

  it("shows persona descriptions", () => {
    render(() => <OnboardingDialog open={true} onComplete={() => {}} />);
    expect(screen.getByText(/Study content created by others/i)).toBeInTheDocument();
    expect(screen.getByText(/Build your own decks/i)).toBeInTheDocument();
    expect(screen.getByText(/Discover, organize, and share/i)).toBeInTheDocument();
  });

  it("Get Started button is disabled until a persona is selected", () => {
    render(() => <OnboardingDialog open={true} onComplete={() => {}} />);
    const button = screen.getByRole("button", { name: /Get Started/i });
    expect(button).toBeDisabled();
  });

  it("enables Get Started button after selecting persona", async () => {
    render(() => <OnboardingDialog open={true} onComplete={() => {}} />);

    const learnerOption = screen.getByText("Learner").closest("button");
    fireEvent.click(learnerOption!);

    const button = screen.getByRole("button", { name: /Get Started/i });
    expect(button).not.toBeDisabled();
  });

  it("calls updatePreferences and onComplete when submitting", async () => {
    const { api } = await import("$lib/api");
    vi.mocked(api.updatePreferences).mockResolvedValue(
      {
        ok: true,
        json: () => Promise.resolve({ persona: "creator", onboarding_completed_at: "2024-01-01" }),
      } as Response,
    );

    const onComplete = vi.fn();
    render(() => <OnboardingDialog open={true} onComplete={onComplete} />);

    const creatorOption = screen.getByText("Creator").closest("button");
    fireEvent.click(creatorOption!);

    const submitButton = screen.getByRole("button", { name: /Get Started/i });
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(api.updatePreferences).toHaveBeenCalledWith({ persona: "creator", complete_onboarding: true });
      expect(onComplete).toHaveBeenCalledWith("creator");
    });
  });

  it("shows submitting state", async () => {
    const { api } = await import("$lib/api");
    vi.mocked(api.updatePreferences).mockImplementation(() =>
      new Promise((resolve) =>
        setTimeout(() => resolve({ ok: true, json: () => Promise.resolve({}) } as Response), 100)
      )
    );

    render(() => <OnboardingDialog open={true} onComplete={() => {}} />);

    const learnerOption = screen.getByText("Learner").closest("button");
    fireEvent.click(learnerOption!);

    const submitButton = screen.getByRole("button", { name: /Get Started/i });
    fireEvent.click(submitButton);

    expect(screen.getByText("Getting Started...")).toBeInTheDocument();
  });
});
