import { authStore } from "$lib/store";
import { cleanup, render, waitFor } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import LoginSuccess from "../LoginSuccess";

const { mockNavigate } = vi.hoisted(() => ({ mockNavigate: vi.fn() }));

vi.mock("$lib/store", () => ({ authStore: { login: vi.fn() } }));
vi.mock("@solidjs/router", () => ({ useNavigate: () => mockNavigate }));

describe("LoginSuccess Page", () => {
  const originalLocation = window.location;

  beforeEach(() => {
    vi.stubGlobal("location", {
      configurable: true,
      enumerable: true,
      value: { hash: "", href: "http://localhost/login/success", assign: vi.fn(), replace: vi.fn() },
    });
    vi.spyOn(window.history, "replaceState");
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    vi.stubGlobal("location", originalLocation);
  });

  it("logs in and redirects on valid tokens", async () => {
    window.location.hash = "#accessJwt=access123&refreshJwt=refresh123&did=did:plc:123&handle=alice.bsky.social";

    render(() => <LoginSuccess />);

    await waitFor(() => {
      expect(authStore.login).toHaveBeenCalledWith({
        accessJwt: "access123",
        refreshJwt: "refresh123",
        did: "did:plc:123",
        handle: "alice.bsky.social",
      });

      expect(window.history.replaceState).toHaveBeenCalledWith(null, "", "/");
      expect(mockNavigate).toHaveBeenCalledWith("/");
    });
  });

  it("handles missing optional parameters (handle fallback)", async () => {
    window.location.hash = "#accessJwt=access123&refreshJwt=&did=did:plc:123";

    render(() => <LoginSuccess />);

    await waitFor(() => {
      expect(authStore.login).toHaveBeenCalledWith({
        accessJwt: "access123",
        refreshJwt: "",
        did: "did:plc:123",
        handle: "did:plc:123",
      });
      expect(mockNavigate).toHaveBeenCalledWith("/");
    });
  });

  it("redirects to error on missing required tokens", async () => {
    window.location.hash = "#did=did:plc:123";

    render(() => <LoginSuccess />);

    await waitFor(() => {
      expect(authStore.login).not.toHaveBeenCalled();
      expect(mockNavigate).toHaveBeenCalledWith("/login?error=missing_tokens");
    });
  });
});
