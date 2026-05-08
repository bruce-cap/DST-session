import { describe, expect, it } from "vitest";
import {
  formatDateTime,
  formatTokenCount,
  getGroupKey,
  groupSessions,
  matchesSession,
  normalizeSession
} from "./session";
import type { SessionRecord } from "../types";

describe("session helpers", () => {
  it("normalizes metadata and extracts the first user text block as preview", () => {
    const record = normalizeSession({
      metadata: {
        id: "a22e6c3d-86bf-4f20-a806-749bd57fed1d",
        title: "你好",
        created_at: "2026-05-07T16:45:29.938084700Z",
        updated_at: "2026-05-07T16:46:29.938084700Z",
        message_count: 2,
        total_tokens: 23876,
        model: "deepseek-v4-flash",
        workspace: "C:\\Users\\Cap",
        mode: "agent"
      },
      messages: [
        {
          role: "user",
          content: [{ type: "text", text: "帮我继续这个会话" }]
        }
      ]
    }, "C:\\Users\\Cap\\.deepseek\\sessions\\a22e6c3d.json");

    expect(record.id).toBe("a22e6c3d-86bf-4f20-a806-749bd57fed1d");
    expect(record.shortId).toBe("a22e6c3d");
    expect(record.title).toBe("你好");
    expect(record.preview).toBe("帮我继续这个会话");
    expect(record.messageCount).toBe(2);
    expect(record.totalTokens).toBe(23876);
  });

  it("falls back to the first user message when metadata title is missing", () => {
    const record = normalizeSession({
      metadata: {
        id: "000a7d86-d0d2-48f0-a8b4-f14dd082b9b4"
      },
      messages: [
        {
          role: "user",
          content: "帮我看下这个项目"
        }
      ]
    }, "C:\\sessions\\000a7d86.json");

    expect(record.title).toBe("帮我看下这个项目");
    expect(record.preview).toBe("帮我看下这个项目");
    expect(record.shortId).toBe("000a7d86");
  });

  it("matches search across title, preview, workspace, model, and id", () => {
    const session = sampleSession({
      title: "帮我看下这个项目",
      preview: "分析 Free-BAI 代码结构",
      workspace: "C:\\Users\\Cap\\Desktop\\Free-BAI-main",
      model: "deepseek-v4-pro"
    });

    expect(matchesSession(session, "Free-BAI")).toBe(true);
    expect(matchesSession(session, "v4-pro")).toBe(true);
    expect(matchesSession(session, "000a7d86")).toBe(true);
    expect(matchesSession(session, "不存在")).toBe(false);
  });

  it("groups sessions by workspace and keeps newest sessions first", () => {
    const sessions = [
      sampleSession({
        id: "b22e6c3d-86bf-4f20-a806-749bd57fed1d",
        updatedAt: "2026-05-07T12:00:00Z",
        workspace: "C:\\repo-a"
      }),
      sampleSession({
        id: "c22e6c3d-86bf-4f20-a806-749bd57fed1d",
        updatedAt: "2026-05-08T12:00:00Z",
        workspace: "C:\\repo-a"
      }),
      sampleSession({
        id: "d22e6c3d-86bf-4f20-a806-749bd57fed1d",
        updatedAt: "2026-05-08T10:00:00Z",
        workspace: "C:\\repo-b"
      })
    ];

    const groups = groupSessions(sessions, "workspace", new Set());

    expect(groups).toHaveLength(2);
    expect(groups[0].key).toBe("C:\\repo-a");
    expect(groups[0].label).toBe("repo-a");
    expect(groups[0].sessions.map((session) => session.id)).toEqual([
      "c22e6c3d-86bf-4f20-a806-749bd57fed1d",
      "b22e6c3d-86bf-4f20-a806-749bd57fed1d"
    ]);
  });

  it("uses the last workspace directory as the visible group label", () => {
    expect(
      getGroupKey(
        sampleSession({ workspace: "C:\\Users\\Example\\Desktop\\Free-BAI-main" }),
        "workspace",
        new Set()
      )
    ).toEqual({
      key: "C:\\Users\\Example\\Desktop\\Free-BAI-main",
      label: "Free-BAI-main"
    });
  });

  it("formats date and token counts for compact display", () => {
    expect(formatDateTime("2026-05-07T16:45:29Z")).toContain("2026");
    expect(formatTokenCount(23876)).toBe("24k");
    expect(getGroupKey(sampleSession({ model: "" }), "model", new Set())).toEqual({
      key: "(no model)",
      label: "(no model)"
    });
  });
});

function sampleSession(overrides: Partial<SessionRecord> = {}): SessionRecord {
  return {
    id: "000a7d86-d0d2-48f0-a8b4-f14dd082b9b4",
    shortId: "000a7d86",
    title: "Untitled",
    preview: "",
    createdAt: "2026-05-07T03:12:37Z",
    updatedAt: "2026-05-07T03:27:00Z",
    messageCount: 1,
    totalTokens: 0,
    model: "deepseek-v4-pro",
    workspace: "C:\\Users\\Cap",
    mode: "agent",
    path: "C:\\sessions\\000a7d86.json",
    invalidReason: null,
    ...overrides
  };
}
