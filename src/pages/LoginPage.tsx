import { FormEvent, useState } from "react";
import { LockKeyhole, LogIn, UserRound } from "lucide-react";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";

type LoginPageProps = {
  onLogin: (username: string, password: string) => void;
};

export function LoginPage({ onLogin }: LoginPageProps) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");

  const submitLogin = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (!username.trim() || !password.trim()) {
      setError("Please enter username and password.");
      return;
    }

    setError("");
    onLogin(username.trim(), password);
  };

  return (
    <main className="grid min-h-screen place-items-center bg-canvas px-6 text-ink">
      <section className="w-full max-w-[420px] rounded-lg border border-stone-200 bg-panel p-6 shadow-sm">
        <div className="flex items-center gap-3">
          <div className="flex h-11 w-11 items-center justify-center rounded-md bg-brand text-white">
            <LockKeyhole className="h-5 w-5" />
          </div>
          <div>
            <h1 className="text-xl font-bold leading-tight">PJ Yuji Statistics</h1>
            <p className="mt-1 text-sm text-slate-500">Sign in to continue.</p>
          </div>
        </div>

        <form className="mt-6 space-y-4" onSubmit={submitLogin}>
          <label className="block">
            <span className="text-xs font-bold text-slate-500">Username</span>
            <div className="mt-1 flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-slate-900 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
              <UserRound className="h-4 w-4 shrink-0 text-slate-400" />
              <InputText
                className="h-full min-w-0 flex-1 border-0 bg-transparent text-sm outline-none"
                autoComplete="username"
                autoFocus
                placeholder="username"
                type="text"
                value={username}
                onChange={(event) => setUsername(event.target.value)}
              />
            </div>
          </label>

          <label className="block">
            <span className="text-xs font-bold text-slate-500">Password</span>
            <div className="mt-1 flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-slate-900 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
              <LockKeyhole className="h-4 w-4 shrink-0 text-slate-400" />
              <InputText
                className="h-full min-w-0 flex-1 border-0 bg-transparent text-sm outline-none"
                autoComplete="current-password"
                placeholder="password"
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
              />
            </div>
          </label>

          {error && (
            <p className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm font-semibold text-red-700">
              {error}
            </p>
          )}

          <Button
            className="flex h-10 w-full items-center justify-center gap-2 rounded-md bg-brand px-3 text-sm font-bold text-white hover:opacity-90"
            type="submit"
          >
            <LogIn className="h-4 w-4" />
            Login
          </Button>
        </form>
      </section>
    </main>
  );
}
