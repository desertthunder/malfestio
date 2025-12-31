import { api } from "$lib/api";
import { authStore } from "$lib/store";
import { cleanup, fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import Login from "../Login";

const { mockNavigate } = vi.hoisted(() => ({ mockNavigate: vi.fn() }));

vi.mock("$lib/api", () => ({ api: { startOAuth: vi.fn(), post: vi.fn() } }));

vi.mock("$lib/store", () => ({ authStore: { login: vi.fn() } }));

vi.mock("@solidjs/router", () => ({ useNavigate: () => mockNavigate }));

vi.mock(
  "$components/layout/AppLayout",
  () => ({ AppLayout: (props: { children: JSX.Element }) => <div data-testid="app-layout">{props.children}</div> }),
);

describe("Login Page", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("renders login form correctly", () => {
    render(() => <Login />);
    expect(screen.getByText("Log in")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("user.bsky.social")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("••••••••")).toBeInTheDocument();
    expect(screen.getByText("Continue with OAuth")).toBeInTheDocument();
  });

  it("switches button text based on password", async () => {
    render(() => <Login />);
    const passwordInput = screen.getByPlaceholderText("••••••••");
    const button = screen.getByRole("button");

    expect(button).toHaveTextContent("Continue with OAuth");

    fireEvent.input(passwordInput, { target: { value: "password123" } });
    expect(button).toHaveTextContent("Log in with Password");

    fireEvent.input(passwordInput, { target: { value: "" } });
    expect(button).toHaveTextContent("Continue with OAuth");
  });

  it("initiates OAuth flow when password is empty", async () => {
    vi.mocked(api.startOAuth).mockResolvedValue({ ok: true } as Response);

    render(() => <Login />);
    const handleInput = screen.getByPlaceholderText("user.bsky.social");
    const button = screen.getByRole("button");

    fireEvent.input(handleInput, { target: { value: "alice.bsky.social" } });
    fireEvent.click(button);

    await waitFor(() => {
      expect(api.startOAuth).toHaveBeenCalledWith("alice.bsky.social");
    });
  });

  it("initiates Legacy flow when password is provided", async () => {
    vi.mocked(api.post).mockResolvedValue(
      { ok: true, json: () => Promise.resolve({ accessJwt: "token", did: "did:plc:123" }) } as Response,
    );

    render(() => <Login />);
    const handleInput = screen.getByPlaceholderText("user.bsky.social");
    const passwordInput = screen.getByPlaceholderText("••••••••");
    const button = screen.getByRole("button");

    fireEvent.input(handleInput, { target: { value: "alice.bsky.social" } });
    fireEvent.input(passwordInput, { target: { value: "password123" } });
    fireEvent.click(button);

    await waitFor(() => {
      expect(api.post).toHaveBeenCalledWith("/auth/login", {
        identifier: "alice.bsky.social",
        password: "password123",
      });
      expect(authStore.login).toHaveBeenCalled();
      expect(mockNavigate).toHaveBeenCalledWith("/");
    });
  });

  it("displays error on OAuth failure", async () => {
    vi.mocked(api.startOAuth).mockResolvedValue(
      { ok: false, json: () => Promise.resolve({ error: "Invalid handle" }) } as Response,
    );

    render(() => <Login />);
    const handleInput = screen.getByPlaceholderText("user.bsky.social");
    const button = screen.getByRole("button");

    fireEvent.input(handleInput, { target: { value: "bad.handle" } });
    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Invalid handle/)).toBeInTheDocument();
    });
  });

  it("displays error on Legacy Login failure", async () => {
    vi.mocked(api.post).mockResolvedValue(
      { ok: false, json: () => Promise.resolve({ error: "Wrong password" }) } as Response,
    );

    render(() => <Login />);
    const handleInput = screen.getByPlaceholderText("user.bsky.social");
    const passwordInput = screen.getByPlaceholderText("••••••••");
    const button = screen.getByRole("button");

    fireEvent.input(handleInput, { target: { value: "alice.bsky.social" } });
    fireEvent.input(passwordInput, { target: { value: "wrongpass" } });
    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Wrong password/)).toBeInTheDocument();
    });
  });
});
