import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import FileDropZone from "../FileDropZone";

describe("FileDropZone", () => {
  afterEach(cleanup);

  it("renders correctly", () => {
    render(() => <FileDropZone onFileSelect={() => {}} />);
    expect(screen.getByText("Click to upload or drag and drop")).toBeInTheDocument();
    expect(screen.getByText("PDF or DOCX (max 10MB)")).toBeInTheDocument();
  });

  it("handles file selection via input change", () => {
    const onFileSelectRequest = vi.fn();
    render(() => <FileDropZone onFileSelect={onFileSelectRequest} />);

    const file = new File(["dummy content"], "test.pdf", { type: "application/pdf" });
    const input = screen.getByTestId("file-upload-input");

    Object.defineProperty(input, "files", { value: [file] });

    fireEvent.change(input);

    expect(onFileSelectRequest).toHaveBeenCalledWith(file);
  });

  it("handles drag and drop", () => {
    const onFileSelectRequest = vi.fn();
    render(() => <FileDropZone onFileSelect={onFileSelectRequest} />);

    const dropZone = screen.getByText("Click to upload or drag and drop").closest("div");
    expect(dropZone).not.toBeNull();

    const file = new File(["dummy content"], "lecture.pdf", { type: "application/pdf" });

    fireEvent.drop(dropZone!, { dataTransfer: { files: [file] } });

    expect(onFileSelectRequest).toHaveBeenCalledWith(file);
  });

  it("validates file size", () => {
    const onFileSelectRequest = vi.fn();
    const onErrorRequest = vi.fn();
    render(() => <FileDropZone onFileSelect={onFileSelectRequest} onError={onErrorRequest} maxSize={10} />); // 10 bytes limit

    const file = new File(["content larger than 10 bytes"], "large.pdf", { type: "application/pdf" });
    const input = screen.getByTestId("file-upload-input");

    Object.defineProperty(input, "files", { value: [file] });
    fireEvent.change(input);

    expect(onFileSelectRequest).not.toHaveBeenCalled();
    expect(onErrorRequest).toHaveBeenCalledWith(expect.stringContaining("File size exceeds limit"));
  });
});
