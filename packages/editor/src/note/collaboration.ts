import { keymap } from "prosemirror-keymap";
import type { Node as PMNode, Schema } from "prosemirror-model";
import type { Plugin } from "prosemirror-state";
import {
  prosemirrorJSONToYXmlFragment,
  redo,
  undo,
  ySyncPlugin,
  yUndoPlugin,
  yXmlFragmentToProseMirrorRootNode,
} from "y-prosemirror";
import type * as Y from "yjs";

type JSONContent = Parameters<typeof prosemirrorJSONToYXmlFragment>[1];

/**
 * Binds the editor to a shared Yjs fragment. The fragment is the source of
 * truth: `initialContent` is ignored once a fragment is supplied and every
 * local transaction is mirrored into it, while remote updates are rendered
 * into the ProseMirror document as regular transactions.
 */
export type NoteCollaboration = {
  fragment: Y.XmlFragment;
};

export const NOTE_FRAGMENT_NAME = "prosemirror";

export function collaborationPlugins(
  collaboration: NoteCollaboration,
): Plugin[] {
  return [
    ySyncPlugin(collaboration.fragment),
    yUndoPlugin(),
    keymap({
      "Mod-z": undo,
      "Mod-y": redo,
      "Mod-Shift-z": redo,
    }),
  ];
}

export function collaborationDoc(
  collaboration: NoteCollaboration,
  schema: Schema,
): PMNode {
  return yXmlFragmentToProseMirrorRootNode(collaboration.fragment, schema);
}

export function isFragmentEmpty(fragment: Y.XmlFragment): boolean {
  return fragment.length === 0;
}

/** Writes a ProseMirror document into an empty fragment. */
export function seedFragment(
  schema: Schema,
  content: JSONContent,
  fragment: Y.XmlFragment,
): void {
  if (!isFragmentEmpty(fragment)) {
    return;
  }
  prosemirrorJSONToYXmlFragment(schema, content, fragment);
}
