import { api } from "$lib/api";
import { authStore } from "$lib/store";
import { Button } from "$ui/Button";
import { createResource, createSignal, For, Show } from "solid-js";

type Comment = {
  id: string;
  deck_id: string;
  author_did: string;
  content: string;
  parent_id: string | null;
  created_at: string;
};

type CommentNode = { comment: Comment; children: CommentNode[] };

type CommentSectionProps = { deckId: string };

function buildTree(comments: Comment[]): CommentNode[] {
  const map = new Map<string, CommentNode>();
  const roots: CommentNode[] = [];

  for (const c of comments) {
    map.set(c.id, { comment: c, children: [] });
  }

  for (const c of comments) {
    if (c.parent_id && map.has(c.parent_id)) {
      map.get(c.parent_id)!.children.push(map.get(c.id)!);
    } else {
      roots.push(map.get(c.id)!);
    }
  }
  return roots;
}

export function CommentSection(props: CommentSectionProps) {
  const [comments, { refetch }] = createResource(async () => {
    const res = await api.getComments(props.deckId);
    if (res.ok) {
      return (await res.json()) as Comment[];
    }
    return [];
  });

  const [newComment, setNewComment] = createSignal("");
  const [replyTo, setReplyTo] = createSignal<string | null>(null);

  const submitComment = async (parentId?: string) => {
    if (!newComment().trim()) return;
    await api.addComment(props.deckId, newComment(), parentId);
    setNewComment("");
    setReplyTo(null);
    refetch();
  };

  const CommentItem = (node: { node: CommentNode }) => (
    <div class="border-l-2 border-gray-200 pl-4 my-2">
      <div class="text-sm font-bold text-gray-600">{node.node.comment.author_did}</div>
      <div class="my-1">{node.node.comment.content}</div>
      <div class="text-xs text-gray-500 flex gap-2">
        <span>{new Date(node.node.comment.created_at).toLocaleString()}</span>
        <button class="text-blue-500 hover:underline" onClick={() => setReplyTo(node.node.comment.id)}>Reply</button>
      </div>

      <Show when={replyTo() === node.node.comment.id}>
        <div class="mt-2 flex gap-2">
          <input
            type="text"
            class="border rounded p-1 flex-1 text-sm"
            value={newComment()}
            onInput={(e) => setNewComment(e.currentTarget.value)}
            placeholder="Write a reply..." />
          <Button size="sm" onClick={() => submitComment(node.node.comment.id)}>Post</Button>
          <Button size="sm" variant="ghost" onClick={() => setReplyTo(null)}>Cancel</Button>
        </div>
      </Show>

      <For each={node.node.children}>{(child) => <CommentItem node={child} />}</For>
    </div>
  );

  return (
    <div class="mt-8">
      <h3 class="text-xl font-bold mb-4">Comments</h3>

      <Show when={authStore.user}>
        <div class="flex gap-2 mb-6">
          <textarea
            class="border rounded p-2 flex-1 w-full"
            rows={2}
            placeholder="Add a comment..."
            value={replyTo() ? "" : newComment()} // Clear if replying elsewhere, actually separate state might be better but simple for now
            onInput={(e) => {
              if (!replyTo()) setNewComment(e.currentTarget.value);
            }} />
          <div class="flex flex-col justify-end">
            <Button onClick={() => submitComment()} disabled={!!replyTo()}>Post</Button>
          </div>
        </div>
      </Show>

      <Show when={comments()} fallback={<div class="animate-pulse">Loading comments...</div>}>
        {(data) => {
          const list = data as unknown as Comment[];
          return (
            <div class="space-y-4">
              <For each={buildTree(list)}>{(node) => <CommentItem node={node} />}</For>
              {list.length === 0 && <div class="text-gray-500 italic">No comments yet.</div>}
            </div>
          );
        }}
      </Show>
    </div>
  );
}
