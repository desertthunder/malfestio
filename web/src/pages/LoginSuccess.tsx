import { authStore } from "$lib/store";
import { useNavigate } from "@solidjs/router";
import { type Component, onMount } from "solid-js";

const LoginSuccess: Component = () => {
  const navigate = useNavigate();

  onMount(() => {
    const hash = window.location.hash.substring(1);
    const params = new URLSearchParams(hash);

    const accessJwt = params.get("accessJwt");
    const refreshJwt = params.get("refreshJwt");
    const did = params.get("did");
    const handle = params.get("handle");

    if (accessJwt && did) {
      authStore.login({ accessJwt, refreshJwt: refreshJwt || "", did, handle: handle || did });
      window.history.replaceState(null, "", "/");
      navigate("/");
    } else {
      console.error("Missing tokens in login success");
      navigate("/login?error=missing_tokens");
    }
  });

  return (
    <div class="flex items-center justify-center h-screen bg-[#161616] text-[#F4F4F4]">
      <div class="flex flex-col items-center gap-4">
        <div class="w-8 h-8 border-t-2 border-[#0F62FE] rounded-full animate-spin" />
        <p class="text-sm text-[#8D8D8D]">Finalizing login...</p>
      </div>
    </div>
  );
};

export default LoginSuccess;
