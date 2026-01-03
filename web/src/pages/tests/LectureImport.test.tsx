import { cleanup, fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, type Mock, vi } from "vitest";
import LectureImport from "../LectureImport";

vi.mock(
  "$components/import/FileDropZone",
  () => ({
    default: (props: { onFileSelect: (f: File) => void }) => (
      <div data-testid="mock-dropzone">
        <button onClick={() => props.onFileSelect(new File(["content"], "test.pdf"))}>Mock Upload</button>
      </div>
    ),
  }),
);

vi.mock("$lib/api", () => ({ api: { createNote: vi.fn(), createDeck: vi.fn() } }));

describe("LectureImport", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    globalThis.fetch = vi.fn();
  });

  afterEach(cleanup);

  it("renders the import page", () => {
    render(() => <LectureImport />);
    expect(screen.getByText("Import Lecture Notes")).toBeInTheDocument();
    expect(screen.getByTestId("mock-dropzone")).toBeInTheDocument();
  });

  it("handles successful file upload and persistence actions", async () => {
    const { api } = await import("$lib/api");
    vi.mocked(api.createNote).mockResolvedValue({ ok: true } as Response);
    vi.mocked(api.createDeck).mockResolvedValue({ ok: true } as Response);

    const mockResponse = {
      filename: "test.pdf",
      content: "Full extracted content",
      chunks: [{ heading: "Abstract", content: "This is the abstract." }, {
        heading: "Introduction",
        content: "Intro content.",
      }],
    };

    (globalThis.fetch as Mock).mockResolvedValue({ ok: true, json: () => Promise.resolve(mockResponse) });

    render(() => <LectureImport />);

    fireEvent.click(screen.getByText("Mock Upload"));

    await waitFor(() => {
      expect(screen.getByText("Extracted Content")).toBeInTheDocument();
      expect(screen.getByText("from test.pdf")).toBeInTheDocument();
    });

    const saveButtons = screen.getAllByTitle("Save this chunk as a note");
    fireEvent.click(saveButtons[0]);
    await waitFor(() => {
      expect(api.createNote).toHaveBeenCalledWith({
        title: "Abstract",
        body: "This is the abstract.",
        tags: ["lecture-import"],
        visibility: { type: "Private" },
      });
    });

    const saveAllButton = screen.getByText("Save All to Notes");
    await waitFor(() => {
      expect(saveAllButton).not.toBeDisabled();
    });

    vi.mocked(api.createNote).mockClear();
    fireEvent.click(saveAllButton);
    await waitFor(() => {
      expect(api.createNote).toHaveBeenCalledTimes(2);
    });

    fireEvent.click(screen.getByText("Create Flashcards"));
    await waitFor(() => {
      expect(api.createDeck).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Flashcards: test.pdf",
          cards: expect.arrayContaining([
            expect.objectContaining({ front: "Abstract", back: "This is the abstract." }),
            expect.objectContaining({ front: "Introduction", back: "Intro content." }),
          ]),
        }),
      );
    });
  });

  it("handles upload error", async () => {
    (globalThis.fetch as Mock).mockResolvedValue({ ok: false, statusText: "Server Error" });

    render(() => <LectureImport />);
    fireEvent.click(screen.getByText("Mock Upload"));

    await waitFor(() => {
      expect(screen.getByText("Upload failed: Server Error")).toBeInTheDocument();
    });
  });
});
