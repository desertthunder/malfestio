import clsx from "clsx";
import { createSignal } from "solid-js";

type FileInputProps = {
  onFileSelect: (file: File) => void;
  onError?: (message: string) => void;
  accept?: string;
  disabled?: boolean;
  maxSize?: number; // in bytes
};

export default function FileDropZone(props: FileInputProps) {
  const [isDragOver, setIsDragOver] = createSignal(false);
  let inputRef: HTMLInputElement | undefined;

  const validateAndSelect = (file: File) => {
    if (props.maxSize && file.size > props.maxSize) {
      props.onError?.(`File size exceeds limit of ${(props.maxSize / (1024 * 1024)).toFixed(0)}MB`);
      return;
    }
    props.onFileSelect(file);
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    if (!props.disabled) {
      setIsDragOver(true);
    }
  };

  const handleDragLeave = () => {
    setIsDragOver(false);
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);

    if (props.disabled) return;

    if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
      validateAndSelect(e.dataTransfer.files[0]);
    }
  };

  const handleClick = () => {
    if (!props.disabled) {
      inputRef?.click();
    }
  };

  const handleInputChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      validateAndSelect(target.files[0]);
    }
  };

  return (
    <div
      class={clsx(
        "border-2 border-dashed rounded-xl p-8 transition-colors cursor-pointer flex flex-col items-center justify-center text-center gap-4",
        isDragOver()
          ? "border-accent-500 bg-accent-500/10"
          : "border-neutral-700 hover:border-neutral-600 bg-neutral-800/50 hover:bg-neutral-800",
        props.disabled && "opacity-50 cursor-not-allowed",
      )}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      onClick={handleClick}>
      <input
        type="file"
        data-testid="file-upload-input"
        ref={inputRef}
        class="hidden"
        accept={props.accept}
        onChange={handleInputChange}
        disabled={props.disabled} />

      {/* TODO: replace with i-* icon */}
      <div class="p-4 rounded-full bg-neutral-700/50">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="32"
          height="32"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="text-neutral-400">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" x2="12" y1="3" y2="15" />
        </svg>
      </div>

      <div class="space-y-1">
        <p class="text-lg font-medium">Click to upload or drag and drop</p>
        <p class="text-sm text-neutral-400">PDF or DOCX (max 10MB)</p>
      </div>
    </div>
  );
}
