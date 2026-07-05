import { useEffect, useState } from "react";
import { api } from "@/lib/tauri";
import type { Masters } from "@/types";

interface UseMastersResult {
  masters: Masters | null;
  loading: boolean;
  error: unknown;
}

/** 起動時にマスタ（22件）を取得する。 */
export function useMasters(): UseMastersResult {
  const [masters, setMasters] = useState<Masters | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<unknown>(null);

  useEffect(() => {
    let cancelled = false;
    api
      .getMasters()
      .then((m) => {
        if (!cancelled) setMasters(m);
      })
      .catch((e) => {
        if (!cancelled) setError(e);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return { masters, loading, error };
}
