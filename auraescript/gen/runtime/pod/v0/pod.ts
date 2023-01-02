/* eslint-disable */

export const protobufPackage = "aurae.runtime.pod.v0";

export interface PodServiceAllocateRequest {
}

export interface PodServiceAllocateResponse {
}

export interface PodServiceFreeRequest {
}

export interface PodServiceFreeResponse {
}

export interface Pod {
}

export interface PodServiceStartRequest {
}

export interface PodServiceStartResponse {
}

export interface PodServiceStopRequest {
}

export interface PodServiceStopResponse {
}

export interface Container {
}

function createBasePodServiceAllocateRequest(): PodServiceAllocateRequest {
  return {};
}

export const PodServiceAllocateRequest = {
  fromJSON(_: any): PodServiceAllocateRequest {
    return {};
  },

  toJSON(_: PodServiceAllocateRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PodServiceAllocateRequest>, I>>(_: I): PodServiceAllocateRequest {
    const message = createBasePodServiceAllocateRequest();
    return message;
  },
};

function createBasePodServiceAllocateResponse(): PodServiceAllocateResponse {
  return {};
}

export const PodServiceAllocateResponse = {
  fromJSON(_: any): PodServiceAllocateResponse {
    return {};
  },

  toJSON(_: PodServiceAllocateResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PodServiceAllocateResponse>, I>>(_: I): PodServiceAllocateResponse {
    const message = createBasePodServiceAllocateResponse();
    return message;
  },
};

function createBasePodServiceFreeRequest(): PodServiceFreeRequest {
  return {};
}

export const PodServiceFreeRequest = {
  fromJSON(_: any): PodServiceFreeRequest {
    return {};
  },

  toJSON(_: PodServiceFreeRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PodServiceFreeRequest>, I>>(_: I): PodServiceFreeRequest {
    const message = createBasePodServiceFreeRequest();
    return message;
  },
};

function createBasePodServiceFreeResponse(): PodServiceFreeResponse {
  return {};
}

export const PodServiceFreeResponse = {
  fromJSON(_: any): PodServiceFreeResponse {
    return {};
  },

  toJSON(_: PodServiceFreeResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PodServiceFreeResponse>, I>>(_: I): PodServiceFreeResponse {
    const message = createBasePodServiceFreeResponse();
    return message;
  },
};

function createBasePod(): Pod {
  return {};
}

export const Pod = {
  fromJSON(_: any): Pod {
    return {};
  },

  toJSON(_: Pod): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Pod>, I>>(_: I): Pod {
    const message = createBasePod();
    return message;
  },
};

function createBasePodServiceStartRequest(): PodServiceStartRequest {
  return {};
}

export const PodServiceStartRequest = {
  fromJSON(_: any): PodServiceStartRequest {
    return {};
  },

  toJSON(_: PodServiceStartRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PodServiceStartRequest>, I>>(_: I): PodServiceStartRequest {
    const message = createBasePodServiceStartRequest();
    return message;
  },
};

function createBasePodServiceStartResponse(): PodServiceStartResponse {
  return {};
}

export const PodServiceStartResponse = {
  fromJSON(_: any): PodServiceStartResponse {
    return {};
  },

  toJSON(_: PodServiceStartResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PodServiceStartResponse>, I>>(_: I): PodServiceStartResponse {
    const message = createBasePodServiceStartResponse();
    return message;
  },
};

function createBasePodServiceStopRequest(): PodServiceStopRequest {
  return {};
}

export const PodServiceStopRequest = {
  fromJSON(_: any): PodServiceStopRequest {
    return {};
  },

  toJSON(_: PodServiceStopRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PodServiceStopRequest>, I>>(_: I): PodServiceStopRequest {
    const message = createBasePodServiceStopRequest();
    return message;
  },
};

function createBasePodServiceStopResponse(): PodServiceStopResponse {
  return {};
}

export const PodServiceStopResponse = {
  fromJSON(_: any): PodServiceStopResponse {
    return {};
  },

  toJSON(_: PodServiceStopResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PodServiceStopResponse>, I>>(_: I): PodServiceStopResponse {
    const message = createBasePodServiceStopResponse();
    return message;
  },
};

function createBaseContainer(): Container {
  return {};
}

export const Container = {
  fromJSON(_: any): Container {
    return {};
  },

  toJSON(_: Container): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Container>, I>>(_: I): Container {
    const message = createBaseContainer();
    return message;
  },
};

/**
 * / A pod is a higher level abstraction than Aurae cells, and to most users
 * / will look at feel like one or more "containers".
 * /
 * / Pods will run an OCI compliant container image.
 * /
 * / A pod is a group of one or more containers with shared network and storage.
 */
export interface PodService {
  allocate(request: PodServiceAllocateRequest): Promise<PodServiceAllocateResponse>;
  start(request: PodServiceStartRequest): Promise<PodServiceStartResponse>;
  stop(request: PodServiceStopRequest): Promise<PodServiceStopResponse>;
  free(request: PodServiceFreeRequest): Promise<PodServiceFreeResponse>;
}

type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;

export type DeepPartial<T> = T extends Builtin ? T
  : T extends Array<infer U> ? Array<DeepPartial<U>> : T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>>
  : T extends {} ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type Exact<P, I extends P> = P extends Builtin ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & { [K in Exclude<keyof I, KeysOfUnion<P>>]: never };
