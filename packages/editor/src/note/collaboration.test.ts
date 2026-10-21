import { EditorState } from "prosemirror-state";
import { describe, expect, it } from "vitest";
import * as Y from "yjs";

import {
  collaborationDoc,
  collaborationPlugins,
  isFragmentEmpty,
  NOTE_FRAGMENT_NAME,
  seedFragment,
} from "./collaboration";
import { schema } from "./schema";

const content = {
  type: "doc",
  content: [
    {
      type: "heading",
      attrs: { level: 1 },
      content: [{ type: "text", text: "Kickoff" }],
    },
    { type: "paragraph", content: [{ type: "text", text: "Agenda" }] },
  ],
};

function fragmentOf(doc: Y.Doc) {
  return doc.getXmlFragment(NOTE_FRAGMENT_NAME);
}

describe("note collaboration", () => {
  it("seeds an empty fragment once and renders it as a ProseMirror doc", () => {
    const doc = new Y.Doc();
    const fragment = fragmentOf(doc);
    expect(isFragmentEmpty(fragment)).toBe(true);

    seedFragment(schema, content, fragment);
    seedFragment(
      schema,
      { type: "doc", content: [{ type: "paragraph" }] },
      fragment,
    );

    expect(isFragmentEmpty(fragment)).toBe(false);
    expect(collaborationDoc({ fragment }, schema).toJSON()).toEqual(content);
  });

  it("converges two peers through Yjs updates", () => {
    const left = new Y.Doc();
    const right = new Y.Doc();
    seedFragment(schema, content, fragmentOf(left));
    Y.applyUpdate(right, Y.encodeStateAsUpdate(left));

    const rightState = EditorState.create({
      doc: collaborationDoc({ fragment: fragmentOf(right) }, schema),
      plugins: collaborationPlugins({ fragment: fragmentOf(right) }),
    });
    expect(rightState.doc.toJSON()).toEqual(content);

    fragmentOf(left).doc?.transact(() => {
      const paragraph = fragmentOf(left).get(1) as Y.XmlElement;
      paragraph.insert(0, [new Y.XmlText("Shared ")]);
    });
    Y.applyUpdate(right, Y.encodeStateAsUpdate(left));

    expect(
      collaborationDoc({ fragment: fragmentOf(right) }, schema).textContent,
    ).toBe("KickoffShared Agenda");
  });
});
