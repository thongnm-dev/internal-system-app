import { useCallback, useEffect, useMemo, useState } from "react";
import type { MenuKey } from "../types/statistics";
import { defaultRoute, routeByKey, routeByPath } from "./routes";

function currentPath() {
  const hash = window.location.hash.replace(/^#/, "");
  return hash.startsWith("/") ? hash : defaultRoute.path;
}

export function useHashRouter() {
  const [path, setPath] = useState(() => currentPath());

  useEffect(() => {
    if (!window.location.hash) {
      window.history.replaceState(null, "", `#${defaultRoute.path}`);
    }

    const syncPath = () => setPath(currentPath());
    window.addEventListener("hashchange", syncPath);
    return () => window.removeEventListener("hashchange", syncPath);
  }, []);

  const route = useMemo(() => routeByPath(path), [path]);

  const navigate = useCallback((key: MenuKey) => {
    const nextRoute = routeByKey(key);
    if (window.location.hash !== `#${nextRoute.path}`) {
      window.location.hash = nextRoute.path;
      return;
    }
    setPath(nextRoute.path);
  }, []);

  return {
    activeMenu: route.key,
    navigate,
    route,
  };
}
