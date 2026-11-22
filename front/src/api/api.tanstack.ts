
  import { queryOptions, infiniteQueryOptions, type QueryClient } from "@tanstack/react-query"
  import type { EndpointByMethod, ApiClient, SuccessStatusCode, ErrorStatusCode, InferResponseByStatus, ApiCallParams } from "./api.client.ts"
  import { errorStatusCodes, TypedStatusError } from "./api.client.ts"

  type EndpointQueryKeyParams = EndpointParameters & {
      _infinite?: boolean;
  };

  /**
   * Hierarchical keys: `["typed-openapi"]` → `["typed-openapi", path]` → `["typed-openapi", path, params]`
   * so `queryKeyFactory.all` / path prefixes work with TanStack partial matching.
   *
   * `options` is `object` so `ApiCallParams` / `ApiRequestOptions` assign without call-site casts
   * (weak-type check would reject empty request bags that only have overrides/withResponse).
   */
  const createQueryKey = (
      id: PropertyKey,
      options?: object,
      infinite?: boolean,
  ): readonly ["typed-openapi", string, EndpointQueryKeyParams?] => {
      const keyId = String(id);
      const src = (options ?? {}) as EndpointParameters;
      const params: EndpointQueryKeyParams = {};
      let hasParams = false;
      if (infinite) {
          params["_infinite"] = infinite;
          hasParams = true;
      }
      if (src.body) {
          params.body = src.body;
          hasParams = true;
      }
      if (src.header) {
          params.header = src.header;
          hasParams = true;
      }
      if (src.path) {
          params.path = src.path;
          hasParams = true;
      }
      if (src.query) {
          params.query = src.query;
          hasParams = true;
      }
      if (src.cookie) {
          params.cookie = src.cookie;
          hasParams = true;
      }
      if (hasParams) {
          return ["typed-openapi", keyId, params] as const;
      }
      return ["typed-openapi", keyId] as const;
  };

  /** Stable query-key factory for cache reads / invalidation outside of `TanstackQueryApiClient` methods. */
  export const queryKeyFactory = {
      all: ["typed-openapi"] as const,
      endpoint: (id: PropertyKey, options?: object, infinite?: boolean) =>
          createQueryKey(id, options, infinite),
  };

  /** Invalidate queries by endpoint path (and optional params). */
  export const invalidateEndpoint = (
      queryClient: QueryClient,
      path: PropertyKey,
      options?: object,
      infinite?: boolean,
  ) => queryClient.invalidateQueries({ queryKey: createQueryKey(path, options, infinite) });

  // <EndpointByMethod.Shorthands>
  export type GetEndpoints = EndpointByMethod["get"];
export type PostEndpoints = EndpointByMethod["post"];
export type PutEndpoints = EndpointByMethod["put"];
export type DeleteEndpoints = EndpointByMethod["delete"];
export type PatchEndpoints = EndpointByMethod["patch"];
  // </EndpointByMethod.Shorthands>

  // <ApiClientTypes>
  export type EndpointParameters = {
      body?: unknown;
      query?: Record<string, unknown>;
      header?: Record<string, unknown>;
      path?: Record<string, unknown>;
      cookie?: Record<string, unknown>;
  };

  type RequiredKeys<T> = {
      [P in keyof T]-?: undefined extends T[P] ? never : P;
  }[keyof T];

  type MaybeOptionalArg<T> = RequiredKeys<T> extends never ? [config?: T] : [config: T];

  type MutationResponse = {
      status: number;
      data: unknown;
  };

  export type TanstackQueryApiClientOptions = {
      /**
       * Pass TanStack Query's AbortSignal to the underlying request.
       *
       * Disabled by default so an abandoned query can finish and populate the
       * cache. Set to true when the request should be cancelled with the query.
       */
      consumeQuerySignal?: boolean;
  };

  type InferResponseData<TEndpoint, TStatusCode> = InferResponseByStatus<TEndpoint, TStatusCode> extends infer TResponse
      ? TResponse extends { data: infer TData }
          ? Extract<TData, {}>
          : never
      : never;

  // </ApiClientTypes>

  // <ApiClient>
  export class TanstackQueryApiClient {
      constructor(
          public client: ApiClient,
          private readonly options: TanstackQueryApiClientOptions = {},
      ) { }


        // <ApiClient.get>
        get<
            Path extends keyof GetEndpoints,
            TEndpoint extends GetEndpoints[Path]
        >(
            path: Path,
            ...params: MaybeOptionalArg<ApiCallParams<TEndpoint>>
        ) {
            const queryKey = createQueryKey(path, params[0]);
            const endpointQueryOptions = params[0]?.queryOptions;
            const endpointParams = { ...(params[0] || {}) } as Record<string, unknown>;
            delete endpointParams["queryOptions"];
            const consumeQuerySignal = endpointQueryOptions?.consumeQuerySignal ?? this.options.consumeQuerySignal;
            const sharedQueryOptions = queryOptions({
                queryFn: async (queryContext) => {
                    const { queryKey } = queryContext;
                    const keyParams = { ...(queryKey[2] || {}) } as Record<string, unknown>;
                    delete keyParams["_infinite"];
                    const requestParams = {
                        ...endpointParams,
                        ...keyParams,
                        ...(consumeQuerySignal
                            ? { overrides: { signal: queryContext.signal } }
                            : {}),
                        withResponse: false as const
                    };
                    const res = await this.client.get(path, requestParams as never);
                    return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                },
                queryKey: queryKey
            });
            const query = {
                /** type-only property if you need easy access to the endpoint params */
                "~endpoint": {} as TEndpoint,
                queryKey,
                queryFn: {} as "You need to pass .queryOptions to the useQuery hook",
                queryOptions: sharedQueryOptions,
                /** Same options for `useSuspenseQuery` / `prefetchQuery` (TanStack v5). */
                suspenseQueryOptions: sharedQueryOptions,
                /**
                 * Infinite query options. Pass `pageParamKey` to merge `pageParam` into `query[pageParamKey]`.
                 * Query key is tagged with `_infinite: true` so it does not collide with regular queries.
                 */
                infiniteQueryOptions: <TPageParam = unknown>(infiniteOpts: {
                    initialPageParam: TPageParam;
                    getNextPageParam: (
                        lastPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        lastPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    getPreviousPageParam?: (
                        firstPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        firstPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    /** When set, `pageParam` is written into `query[pageParamKey]` before each fetch. */
                    pageParamKey?: string;
                }) => {
                    const infiniteKey = createQueryKey(path, params[0], true);
                    return infiniteQueryOptions({
                        queryKey: infiniteKey,
                        initialPageParam: infiniteOpts.initialPageParam,
                        getNextPageParam: infiniteOpts.getNextPageParam,
                        ...(infiniteOpts.getPreviousPageParam
                            ? { getPreviousPageParam: infiniteOpts.getPreviousPageParam }
                            : {}),
                        queryFn: async (queryContext) => {
                            const { pageParam } = queryContext;
                            const base = { ...endpointParams } as Record<string, unknown>;
                            const query = {
                                ...((base["query"] as Record<string, unknown> | undefined) || {}),
                            };
                            if (infiniteOpts.pageParamKey) {
                                query[infiniteOpts.pageParamKey] = pageParam;
                            }
                            const requestParams = {
                                ...base,
                                ...(Object.keys(query).length ? { query } : {}),
                                ...(consumeQuerySignal
                                    ? { overrides: { signal: queryContext.signal } }
                                    : {}),
                                withResponse: false as const,
                            };
                            const res = await this.client.get(path, requestParams as never);
                            return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                        },
                    });
                },
                /** Invalidate this endpoint's regular (non-infinite) queries. */
                invalidate: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey }),
                /** Invalidate this endpoint's infinite queries. */
                invalidateInfinite: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey: createQueryKey(path, params[0], true) }),
            };

            return query
        }
        // </ApiClient.get>


        // <ApiClient.post>
        post<
            Path extends keyof PostEndpoints,
            TEndpoint extends PostEndpoints[Path]
        >(
            path: Path,
            ...params: MaybeOptionalArg<ApiCallParams<TEndpoint>>
        ) {
            const queryKey = createQueryKey(path, params[0]);
            const endpointQueryOptions = params[0]?.queryOptions;
            const endpointParams = { ...(params[0] || {}) } as Record<string, unknown>;
            delete endpointParams["queryOptions"];
            const consumeQuerySignal = endpointQueryOptions?.consumeQuerySignal ?? this.options.consumeQuerySignal;
            const sharedQueryOptions = queryOptions({
                queryFn: async (queryContext) => {
                    const { queryKey } = queryContext;
                    const keyParams = { ...(queryKey[2] || {}) } as Record<string, unknown>;
                    delete keyParams["_infinite"];
                    const requestParams = {
                        ...endpointParams,
                        ...keyParams,
                        ...(consumeQuerySignal
                            ? { overrides: { signal: queryContext.signal } }
                            : {}),
                        withResponse: false as const
                    };
                    const res = await this.client.post(path, requestParams as never);
                    return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                },
                queryKey: queryKey
            });
            const query = {
                /** type-only property if you need easy access to the endpoint params */
                "~endpoint": {} as TEndpoint,
                queryKey,
                queryFn: {} as "You need to pass .queryOptions to the useQuery hook",
                queryOptions: sharedQueryOptions,
                /** Same options for `useSuspenseQuery` / `prefetchQuery` (TanStack v5). */
                suspenseQueryOptions: sharedQueryOptions,
                /**
                 * Infinite query options. Pass `pageParamKey` to merge `pageParam` into `query[pageParamKey]`.
                 * Query key is tagged with `_infinite: true` so it does not collide with regular queries.
                 */
                infiniteQueryOptions: <TPageParam = unknown>(infiniteOpts: {
                    initialPageParam: TPageParam;
                    getNextPageParam: (
                        lastPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        lastPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    getPreviousPageParam?: (
                        firstPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        firstPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    /** When set, `pageParam` is written into `query[pageParamKey]` before each fetch. */
                    pageParamKey?: string;
                }) => {
                    const infiniteKey = createQueryKey(path, params[0], true);
                    return infiniteQueryOptions({
                        queryKey: infiniteKey,
                        initialPageParam: infiniteOpts.initialPageParam,
                        getNextPageParam: infiniteOpts.getNextPageParam,
                        ...(infiniteOpts.getPreviousPageParam
                            ? { getPreviousPageParam: infiniteOpts.getPreviousPageParam }
                            : {}),
                        queryFn: async (queryContext) => {
                            const { pageParam } = queryContext;
                            const base = { ...endpointParams } as Record<string, unknown>;
                            const query = {
                                ...((base["query"] as Record<string, unknown> | undefined) || {}),
                            };
                            if (infiniteOpts.pageParamKey) {
                                query[infiniteOpts.pageParamKey] = pageParam;
                            }
                            const requestParams = {
                                ...base,
                                ...(Object.keys(query).length ? { query } : {}),
                                ...(consumeQuerySignal
                                    ? { overrides: { signal: queryContext.signal } }
                                    : {}),
                                withResponse: false as const,
                            };
                            const res = await this.client.post(path, requestParams as never);
                            return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                        },
                    });
                },
                /** Invalidate this endpoint's regular (non-infinite) queries. */
                invalidate: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey }),
                /** Invalidate this endpoint's infinite queries. */
                invalidateInfinite: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey: createQueryKey(path, params[0], true) }),
            };

            return query
        }
        // </ApiClient.post>


        // <ApiClient.put>
        put<
            Path extends keyof PutEndpoints,
            TEndpoint extends PutEndpoints[Path]
        >(
            path: Path,
            ...params: MaybeOptionalArg<ApiCallParams<TEndpoint>>
        ) {
            const queryKey = createQueryKey(path, params[0]);
            const endpointQueryOptions = params[0]?.queryOptions;
            const endpointParams = { ...(params[0] || {}) } as Record<string, unknown>;
            delete endpointParams["queryOptions"];
            const consumeQuerySignal = endpointQueryOptions?.consumeQuerySignal ?? this.options.consumeQuerySignal;
            const sharedQueryOptions = queryOptions({
                queryFn: async (queryContext) => {
                    const { queryKey } = queryContext;
                    const keyParams = { ...(queryKey[2] || {}) } as Record<string, unknown>;
                    delete keyParams["_infinite"];
                    const requestParams = {
                        ...endpointParams,
                        ...keyParams,
                        ...(consumeQuerySignal
                            ? { overrides: { signal: queryContext.signal } }
                            : {}),
                        withResponse: false as const
                    };
                    const res = await this.client.put(path, requestParams as never);
                    return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                },
                queryKey: queryKey
            });
            const query = {
                /** type-only property if you need easy access to the endpoint params */
                "~endpoint": {} as TEndpoint,
                queryKey,
                queryFn: {} as "You need to pass .queryOptions to the useQuery hook",
                queryOptions: sharedQueryOptions,
                /** Same options for `useSuspenseQuery` / `prefetchQuery` (TanStack v5). */
                suspenseQueryOptions: sharedQueryOptions,
                /**
                 * Infinite query options. Pass `pageParamKey` to merge `pageParam` into `query[pageParamKey]`.
                 * Query key is tagged with `_infinite: true` so it does not collide with regular queries.
                 */
                infiniteQueryOptions: <TPageParam = unknown>(infiniteOpts: {
                    initialPageParam: TPageParam;
                    getNextPageParam: (
                        lastPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        lastPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    getPreviousPageParam?: (
                        firstPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        firstPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    /** When set, `pageParam` is written into `query[pageParamKey]` before each fetch. */
                    pageParamKey?: string;
                }) => {
                    const infiniteKey = createQueryKey(path, params[0], true);
                    return infiniteQueryOptions({
                        queryKey: infiniteKey,
                        initialPageParam: infiniteOpts.initialPageParam,
                        getNextPageParam: infiniteOpts.getNextPageParam,
                        ...(infiniteOpts.getPreviousPageParam
                            ? { getPreviousPageParam: infiniteOpts.getPreviousPageParam }
                            : {}),
                        queryFn: async (queryContext) => {
                            const { pageParam } = queryContext;
                            const base = { ...endpointParams } as Record<string, unknown>;
                            const query = {
                                ...((base["query"] as Record<string, unknown> | undefined) || {}),
                            };
                            if (infiniteOpts.pageParamKey) {
                                query[infiniteOpts.pageParamKey] = pageParam;
                            }
                            const requestParams = {
                                ...base,
                                ...(Object.keys(query).length ? { query } : {}),
                                ...(consumeQuerySignal
                                    ? { overrides: { signal: queryContext.signal } }
                                    : {}),
                                withResponse: false as const,
                            };
                            const res = await this.client.put(path, requestParams as never);
                            return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                        },
                    });
                },
                /** Invalidate this endpoint's regular (non-infinite) queries. */
                invalidate: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey }),
                /** Invalidate this endpoint's infinite queries. */
                invalidateInfinite: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey: createQueryKey(path, params[0], true) }),
            };

            return query
        }
        // </ApiClient.put>


        // <ApiClient.delete>
        delete<
            Path extends keyof DeleteEndpoints,
            TEndpoint extends DeleteEndpoints[Path]
        >(
            path: Path,
            ...params: MaybeOptionalArg<ApiCallParams<TEndpoint>>
        ) {
            const queryKey = createQueryKey(path, params[0]);
            const endpointQueryOptions = params[0]?.queryOptions;
            const endpointParams = { ...(params[0] || {}) } as Record<string, unknown>;
            delete endpointParams["queryOptions"];
            const consumeQuerySignal = endpointQueryOptions?.consumeQuerySignal ?? this.options.consumeQuerySignal;
            const sharedQueryOptions = queryOptions({
                queryFn: async (queryContext) => {
                    const { queryKey } = queryContext;
                    const keyParams = { ...(queryKey[2] || {}) } as Record<string, unknown>;
                    delete keyParams["_infinite"];
                    const requestParams = {
                        ...endpointParams,
                        ...keyParams,
                        ...(consumeQuerySignal
                            ? { overrides: { signal: queryContext.signal } }
                            : {}),
                        withResponse: false as const
                    };
                    const res = await this.client.delete(path, requestParams as never);
                    return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                },
                queryKey: queryKey
            });
            const query = {
                /** type-only property if you need easy access to the endpoint params */
                "~endpoint": {} as TEndpoint,
                queryKey,
                queryFn: {} as "You need to pass .queryOptions to the useQuery hook",
                queryOptions: sharedQueryOptions,
                /** Same options for `useSuspenseQuery` / `prefetchQuery` (TanStack v5). */
                suspenseQueryOptions: sharedQueryOptions,
                /**
                 * Infinite query options. Pass `pageParamKey` to merge `pageParam` into `query[pageParamKey]`.
                 * Query key is tagged with `_infinite: true` so it does not collide with regular queries.
                 */
                infiniteQueryOptions: <TPageParam = unknown>(infiniteOpts: {
                    initialPageParam: TPageParam;
                    getNextPageParam: (
                        lastPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        lastPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    getPreviousPageParam?: (
                        firstPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        firstPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    /** When set, `pageParam` is written into `query[pageParamKey]` before each fetch. */
                    pageParamKey?: string;
                }) => {
                    const infiniteKey = createQueryKey(path, params[0], true);
                    return infiniteQueryOptions({
                        queryKey: infiniteKey,
                        initialPageParam: infiniteOpts.initialPageParam,
                        getNextPageParam: infiniteOpts.getNextPageParam,
                        ...(infiniteOpts.getPreviousPageParam
                            ? { getPreviousPageParam: infiniteOpts.getPreviousPageParam }
                            : {}),
                        queryFn: async (queryContext) => {
                            const { pageParam } = queryContext;
                            const base = { ...endpointParams } as Record<string, unknown>;
                            const query = {
                                ...((base["query"] as Record<string, unknown> | undefined) || {}),
                            };
                            if (infiniteOpts.pageParamKey) {
                                query[infiniteOpts.pageParamKey] = pageParam;
                            }
                            const requestParams = {
                                ...base,
                                ...(Object.keys(query).length ? { query } : {}),
                                ...(consumeQuerySignal
                                    ? { overrides: { signal: queryContext.signal } }
                                    : {}),
                                withResponse: false as const,
                            };
                            const res = await this.client.delete(path, requestParams as never);
                            return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                        },
                    });
                },
                /** Invalidate this endpoint's regular (non-infinite) queries. */
                invalidate: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey }),
                /** Invalidate this endpoint's infinite queries. */
                invalidateInfinite: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey: createQueryKey(path, params[0], true) }),
            };

            return query
        }
        // </ApiClient.delete>


        // <ApiClient.patch>
        patch<
            Path extends keyof PatchEndpoints,
            TEndpoint extends PatchEndpoints[Path]
        >(
            path: Path,
            ...params: MaybeOptionalArg<ApiCallParams<TEndpoint>>
        ) {
            const queryKey = createQueryKey(path, params[0]);
            const endpointQueryOptions = params[0]?.queryOptions;
            const endpointParams = { ...(params[0] || {}) } as Record<string, unknown>;
            delete endpointParams["queryOptions"];
            const consumeQuerySignal = endpointQueryOptions?.consumeQuerySignal ?? this.options.consumeQuerySignal;
            const sharedQueryOptions = queryOptions({
                queryFn: async (queryContext) => {
                    const { queryKey } = queryContext;
                    const keyParams = { ...(queryKey[2] || {}) } as Record<string, unknown>;
                    delete keyParams["_infinite"];
                    const requestParams = {
                        ...endpointParams,
                        ...keyParams,
                        ...(consumeQuerySignal
                            ? { overrides: { signal: queryContext.signal } }
                            : {}),
                        withResponse: false as const
                    };
                    const res = await this.client.patch(path, requestParams as never);
                    return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                },
                queryKey: queryKey
            });
            const query = {
                /** type-only property if you need easy access to the endpoint params */
                "~endpoint": {} as TEndpoint,
                queryKey,
                queryFn: {} as "You need to pass .queryOptions to the useQuery hook",
                queryOptions: sharedQueryOptions,
                /** Same options for `useSuspenseQuery` / `prefetchQuery` (TanStack v5). */
                suspenseQueryOptions: sharedQueryOptions,
                /**
                 * Infinite query options. Pass `pageParamKey` to merge `pageParam` into `query[pageParamKey]`.
                 * Query key is tagged with `_infinite: true` so it does not collide with regular queries.
                 */
                infiniteQueryOptions: <TPageParam = unknown>(infiniteOpts: {
                    initialPageParam: TPageParam;
                    getNextPageParam: (
                        lastPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        lastPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    getPreviousPageParam?: (
                        firstPage: InferResponseData<TEndpoint, SuccessStatusCode>,
                        allPages: InferResponseData<TEndpoint, SuccessStatusCode>[],
                        firstPageParam: TPageParam,
                        allPageParams: TPageParam[],
                    ) => TPageParam | undefined | null;
                    /** When set, `pageParam` is written into `query[pageParamKey]` before each fetch. */
                    pageParamKey?: string;
                }) => {
                    const infiniteKey = createQueryKey(path, params[0], true);
                    return infiniteQueryOptions({
                        queryKey: infiniteKey,
                        initialPageParam: infiniteOpts.initialPageParam,
                        getNextPageParam: infiniteOpts.getNextPageParam,
                        ...(infiniteOpts.getPreviousPageParam
                            ? { getPreviousPageParam: infiniteOpts.getPreviousPageParam }
                            : {}),
                        queryFn: async (queryContext) => {
                            const { pageParam } = queryContext;
                            const base = { ...endpointParams } as Record<string, unknown>;
                            const query = {
                                ...((base["query"] as Record<string, unknown> | undefined) || {}),
                            };
                            if (infiniteOpts.pageParamKey) {
                                query[infiniteOpts.pageParamKey] = pageParam;
                            }
                            const requestParams = {
                                ...base,
                                ...(Object.keys(query).length ? { query } : {}),
                                ...(consumeQuerySignal
                                    ? { overrides: { signal: queryContext.signal } }
                                    : {}),
                                withResponse: false as const,
                            };
                            const res = await this.client.patch(path, requestParams as never);
                            return res as InferResponseData<TEndpoint, SuccessStatusCode>;
                        },
                    });
                },
                /** Invalidate this endpoint's regular (non-infinite) queries. */
                invalidate: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey }),
                /** Invalidate this endpoint's infinite queries. */
                invalidateInfinite: (queryClient: QueryClient) =>
                    queryClient.invalidateQueries({ queryKey: createQueryKey(path, params[0], true) }),
            };

            return query
        }
        // </ApiClient.patch>


        // <ApiClient.request>
        /**
         * Generic mutation method with full type-safety for any endpoint; it doesnt require parameters to be passed initially
         * but instead will require them to be passed when calling the mutation.mutate() method
         */
        mutation<
            TMethod extends keyof EndpointByMethod,
            TPath extends keyof EndpointByMethod[TMethod],
            TEndpoint extends EndpointByMethod[TMethod][TPath],
            TWithResponse extends boolean = false,
            TSelection = TWithResponse extends true
                ? InferResponseByStatus<TEndpoint, SuccessStatusCode>
                : InferResponseData<TEndpoint, SuccessStatusCode>,
            TError = TEndpoint extends { responses: infer TResponses }
                ? TResponses extends Record<string | number, unknown>
                    ? TypedStatusError<InferResponseData<TEndpoint, ErrorStatusCode>>
                    : Error
                : Error
        >(method: TMethod, path: TPath, options?: {
            withResponse?: TWithResponse;
            selectFn?: (res: TWithResponse extends true
                ? InferResponseByStatus<TEndpoint, SuccessStatusCode>
                : InferResponseData<TEndpoint, SuccessStatusCode>
            ) => TSelection;
             throwOnStatusError?: boolean
             throwOnError?: boolean | ((error: TError) => boolean)
        }) {
            const mutationKey = [{ method, path }] as const;
            const mutationFn = async (params: ApiCallParams<TEndpoint>): Promise<TSelection> => {
                const withResponse = options?.withResponse ?? false;
                const throwOnStatusError = params.throwOnStatusError ?? options?.throwOnStatusError ?? (withResponse ? false : true);
                const selectFn = options?.selectFn;
                const response = await this.client.request(method, path, {
                ...params,
                withResponse: true,
                throwOnStatusError: false,
            } as never) as MutationResponse;

                if (throwOnStatusError && (errorStatusCodes as readonly number[]).includes(response.status)) {
                    throw new TypedStatusError(response as never);
                }

                const finalResponse = withResponse ? response : response.data;
                const res = selectFn
                    ? selectFn(finalResponse as TWithResponse extends true
                        ? InferResponseByStatus<TEndpoint, SuccessStatusCode>
                        : InferResponseData<TEndpoint, SuccessStatusCode>)
                    : finalResponse;
                return res as TSelection;
            };
            return {
            /** type-only property if you need easy access to the endpoint params */
            "~endpoint": {} as TEndpoint,
            mutationKey: mutationKey,
            mutationFn: {} as "You need to pass .mutationOptions to the useMutation hook",
            mutationOptions: {
                throwOnError: options?.throwOnError as boolean | ((error: TError) => boolean),
                mutationKey: mutationKey,
                mutationFn: mutationFn,
            } as Omit<import("@tanstack/react-query").UseMutationOptions<
                TSelection,
                TError,
                ApiCallParams<TEndpoint>
            >, "mutationFn"> & {
                mutationFn: typeof mutationFn
            },
            /** Invalidate cached queries for this endpoint path (not the mutation cache). */
            invalidateQueries: (queryClient: QueryClient) =>
                queryClient.invalidateQueries({ queryKey: createQueryKey(path) }),
        }
    }
        // </ApiClient.request>
  }
