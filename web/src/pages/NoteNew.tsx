import { useNavigate } from "@solidjs/router";
import { NoteEditor } from "../components/NoteEditor";

const NoteNew = () => {
  return (
    <div class="max-w-4xl mx-auto">
      <NoteEditor />
    </div>
  );
};

export default NoteNew;
