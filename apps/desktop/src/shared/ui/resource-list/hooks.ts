import { useQuery } from "@tanstack/react-query";

export function useWebResources<T>(endpoint: string) {
  return useQuery<T[]>({
    queryKey: ["settings", endpoint, "suggestions"],
    queryFn: async () => [],
    initialData: [],
    enabled: false,
  });
}
