import { createContext, type ReactNode, useContext, useMemo, useReducer } from "react";

type AuthUser = {
  username: string;
};

type AuthState = {
  isAuthenticated: boolean;
  returnPath: string | null;
  user: AuthUser | null;
};

type LoginPayload = {
  username: string;
};

type AuthAction =
  | { type: "LOGIN"; payload: LoginPayload }
  | { type: "LOGOUT" }
  | { type: "SET_RETURN_PATH"; payload: string | null };

type AuthContextValue = AuthState & {
  login: (payload: LoginPayload) => void;
  logout: () => void;
  setReturnPath: (path: string | null) => void;
};

const authStorageKey = "pjjyuji.auth.session";

const initialState: AuthState = {
  isAuthenticated: false,
  returnPath: null,
  user: null,
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(authReducer, initialState, loadAuthState);

  const value = useMemo<AuthContextValue>(
    () => ({
      ...state,
      login: (payload) => dispatch({ type: "LOGIN", payload }),
      logout: () => dispatch({ type: "LOGOUT" }),
      setReturnPath: (path) => dispatch({ type: "SET_RETURN_PATH", payload: path }),
    }),
    [state],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuthStore() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuthStore must be used inside AuthProvider.");
  }
  return context;
}

function authReducer(state: AuthState, action: AuthAction): AuthState {
  switch (action.type) {
    case "LOGIN": {
      const nextState = {
        ...state,
        isAuthenticated: true,
        user: {
          username: action.payload.username,
        },
      };
      persistAuthState(nextState);
      return nextState;
    }
    case "LOGOUT":
      window.localStorage.removeItem(authStorageKey);
      return {
        ...initialState,
        returnPath: state.returnPath,
      };
    case "SET_RETURN_PATH":
      if (state.returnPath === action.payload) {
        return state;
      }
      return {
        ...state,
        returnPath: action.payload,
      };
    default:
      return state;
  }
}

function loadAuthState(): AuthState {
  try {
    const saved = window.localStorage.getItem(authStorageKey);
    if (!saved) {
      return initialState;
    }

    const parsed = JSON.parse(saved) as Partial<AuthState>;
    if (!parsed.user?.username) {
      return initialState;
    }

    return {
      ...initialState,
      isAuthenticated: true,
      user: {
        username: parsed.user.username,
      },
    };
  } catch {
    return initialState;
  }
}

function persistAuthState(state: AuthState) {
  window.localStorage.setItem(
    authStorageKey,
    JSON.stringify({
      isAuthenticated: state.isAuthenticated,
      user: state.user,
    }),
  );
}
