/* eslint-disable */

export const protobufPackage = "runtime";

/** / The most primitive workload in Aurae, a standard executable process. */
export interface Executable {
  name: string;
  command: string;
  description: string;
  cellName: string;
}

/** / A reference to an executable and it's parent cell. */
export interface ExecutableReference {
  cellName: string;
  executableName: string;
}

/** / An isolation resource used to divide a system into smaller resource boundaries. */
export interface Cell {
  /**
   * / Resource parameters for control groups (cgroups)
   * / Build on the [cgroups-rs](https://github.com/kata-containers/cgroups-rs) crate.
   * / See [examples](https://github.com/kata-containers/cgroups-rs/blob/main/tests/builder.rs)
   */
  name: string;
  cpus: string;
  mems: string;
  shares: number;
  quota: number;
  /**
   * / Linux namespaces to share with the calling process.
   * / If all values are set to false, the resulting cell
   * / will be as isolated as possible.
   * /
   * / Each shared namespace is a potential security risk.
   */
  nsShareMount: boolean;
  nsShareUts: boolean;
  nsShareIpc: boolean;
  nsSharePid: boolean;
  nsShareNet: boolean;
  nsShareCgroup: boolean;
}

export interface AllocateCellRequest {
  cell: Cell | undefined;
}

export interface AllocateCellResponse {
}

export interface FreeCellRequest {
  cell: Cell | undefined;
}

export interface FreeCellResponse {
}

export interface StartCellRequest {
  executable: Executable | undefined;
}

export interface StartCellResponse {
}

export interface StopCellRequest {
  executableReference: ExecutableReference | undefined;
}

export interface StopCellResponse {
}

function createBaseExecutable(): Executable {
  return { name: "", command: "", description: "", cellName: "" };
}

export const Executable = {
  fromJSON(object: any): Executable {
    return {
      name: isSet(object.name) ? String(object.name) : "",
      command: isSet(object.command) ? String(object.command) : "",
      description: isSet(object.description) ? String(object.description) : "",
      cellName: isSet(object.cellName) ? String(object.cellName) : "",
    };
  },

  toJSON(message: Executable): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.command !== undefined && (obj.command = message.command);
    message.description !== undefined && (obj.description = message.description);
    message.cellName !== undefined && (obj.cellName = message.cellName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Executable>, I>>(object: I): Executable {
    const message = createBaseExecutable();
    message.name = object.name ?? "";
    message.command = object.command ?? "";
    message.description = object.description ?? "";
    message.cellName = object.cellName ?? "";
    return message;
  },
};

function createBaseExecutableReference(): ExecutableReference {
  return { cellName: "", executableName: "" };
}

export const ExecutableReference = {
  fromJSON(object: any): ExecutableReference {
    return {
      cellName: isSet(object.cellName) ? String(object.cellName) : "",
      executableName: isSet(object.executableName) ? String(object.executableName) : "",
    };
  },

  toJSON(message: ExecutableReference): unknown {
    const obj: any = {};
    message.cellName !== undefined && (obj.cellName = message.cellName);
    message.executableName !== undefined && (obj.executableName = message.executableName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ExecutableReference>, I>>(object: I): ExecutableReference {
    const message = createBaseExecutableReference();
    message.cellName = object.cellName ?? "";
    message.executableName = object.executableName ?? "";
    return message;
  },
};

function createBaseCell(): Cell {
  return {
    name: "",
    cpus: "",
    mems: "",
    shares: 0,
    quota: 0,
    nsShareMount: false,
    nsShareUts: false,
    nsShareIpc: false,
    nsSharePid: false,
    nsShareNet: false,
    nsShareCgroup: false,
  };
}

export const Cell = {
  fromJSON(object: any): Cell {
    return {
      name: isSet(object.name) ? String(object.name) : "",
      cpus: isSet(object.cpus) ? String(object.cpus) : "",
      mems: isSet(object.mems) ? String(object.mems) : "",
      shares: isSet(object.shares) ? Number(object.shares) : 0,
      quota: isSet(object.quota) ? Number(object.quota) : 0,
      nsShareMount: isSet(object.nsShareMount) ? Boolean(object.nsShareMount) : false,
      nsShareUts: isSet(object.nsShareUts) ? Boolean(object.nsShareUts) : false,
      nsShareIpc: isSet(object.nsShareIpc) ? Boolean(object.nsShareIpc) : false,
      nsSharePid: isSet(object.nsSharePid) ? Boolean(object.nsSharePid) : false,
      nsShareNet: isSet(object.nsShareNet) ? Boolean(object.nsShareNet) : false,
      nsShareCgroup: isSet(object.nsShareCgroup) ? Boolean(object.nsShareCgroup) : false,
    };
  },

  toJSON(message: Cell): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.cpus !== undefined && (obj.cpus = message.cpus);
    message.mems !== undefined && (obj.mems = message.mems);
    message.shares !== undefined && (obj.shares = Math.round(message.shares));
    message.quota !== undefined && (obj.quota = Math.round(message.quota));
    message.nsShareMount !== undefined && (obj.nsShareMount = message.nsShareMount);
    message.nsShareUts !== undefined && (obj.nsShareUts = message.nsShareUts);
    message.nsShareIpc !== undefined && (obj.nsShareIpc = message.nsShareIpc);
    message.nsSharePid !== undefined && (obj.nsSharePid = message.nsSharePid);
    message.nsShareNet !== undefined && (obj.nsShareNet = message.nsShareNet);
    message.nsShareCgroup !== undefined && (obj.nsShareCgroup = message.nsShareCgroup);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Cell>, I>>(object: I): Cell {
    const message = createBaseCell();
    message.name = object.name ?? "";
    message.cpus = object.cpus ?? "";
    message.mems = object.mems ?? "";
    message.shares = object.shares ?? 0;
    message.quota = object.quota ?? 0;
    message.nsShareMount = object.nsShareMount ?? false;
    message.nsShareUts = object.nsShareUts ?? false;
    message.nsShareIpc = object.nsShareIpc ?? false;
    message.nsSharePid = object.nsSharePid ?? false;
    message.nsShareNet = object.nsShareNet ?? false;
    message.nsShareCgroup = object.nsShareCgroup ?? false;
    return message;
  },
};

function createBaseAllocateCellRequest(): AllocateCellRequest {
  return { cell: undefined };
}

export const AllocateCellRequest = {
  fromJSON(object: any): AllocateCellRequest {
    return { cell: isSet(object.cell) ? Cell.fromJSON(object.cell) : undefined };
  },

  toJSON(message: AllocateCellRequest): unknown {
    const obj: any = {};
    message.cell !== undefined && (obj.cell = message.cell ? Cell.toJSON(message.cell) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<AllocateCellRequest>, I>>(object: I): AllocateCellRequest {
    const message = createBaseAllocateCellRequest();
    message.cell = (object.cell !== undefined && object.cell !== null) ? Cell.fromPartial(object.cell) : undefined;
    return message;
  },
};

function createBaseAllocateCellResponse(): AllocateCellResponse {
  return {};
}

export const AllocateCellResponse = {
  fromJSON(_: any): AllocateCellResponse {
    return {};
  },

  toJSON(_: AllocateCellResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<AllocateCellResponse>, I>>(_: I): AllocateCellResponse {
    const message = createBaseAllocateCellResponse();
    return message;
  },
};

function createBaseFreeCellRequest(): FreeCellRequest {
  return { cell: undefined };
}

export const FreeCellRequest = {
  fromJSON(object: any): FreeCellRequest {
    return { cell: isSet(object.cell) ? Cell.fromJSON(object.cell) : undefined };
  },

  toJSON(message: FreeCellRequest): unknown {
    const obj: any = {};
    message.cell !== undefined && (obj.cell = message.cell ? Cell.toJSON(message.cell) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FreeCellRequest>, I>>(object: I): FreeCellRequest {
    const message = createBaseFreeCellRequest();
    message.cell = (object.cell !== undefined && object.cell !== null) ? Cell.fromPartial(object.cell) : undefined;
    return message;
  },
};

function createBaseFreeCellResponse(): FreeCellResponse {
  return {};
}

export const FreeCellResponse = {
  fromJSON(_: any): FreeCellResponse {
    return {};
  },

  toJSON(_: FreeCellResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FreeCellResponse>, I>>(_: I): FreeCellResponse {
    const message = createBaseFreeCellResponse();
    return message;
  },
};

function createBaseStartCellRequest(): StartCellRequest {
  return { executable: undefined };
}

export const StartCellRequest = {
  fromJSON(object: any): StartCellRequest {
    return { executable: isSet(object.executable) ? Executable.fromJSON(object.executable) : undefined };
  },

  toJSON(message: StartCellRequest): unknown {
    const obj: any = {};
    message.executable !== undefined &&
      (obj.executable = message.executable ? Executable.toJSON(message.executable) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StartCellRequest>, I>>(object: I): StartCellRequest {
    const message = createBaseStartCellRequest();
    message.executable = (object.executable !== undefined && object.executable !== null)
      ? Executable.fromPartial(object.executable)
      : undefined;
    return message;
  },
};

function createBaseStartCellResponse(): StartCellResponse {
  return {};
}

export const StartCellResponse = {
  fromJSON(_: any): StartCellResponse {
    return {};
  },

  toJSON(_: StartCellResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StartCellResponse>, I>>(_: I): StartCellResponse {
    const message = createBaseStartCellResponse();
    return message;
  },
};

function createBaseStopCellRequest(): StopCellRequest {
  return { executableReference: undefined };
}

export const StopCellRequest = {
  fromJSON(object: any): StopCellRequest {
    return {
      executableReference: isSet(object.executableReference)
        ? ExecutableReference.fromJSON(object.executableReference)
        : undefined,
    };
  },

  toJSON(message: StopCellRequest): unknown {
    const obj: any = {};
    message.executableReference !== undefined && (obj.executableReference = message.executableReference
      ? ExecutableReference.toJSON(message.executableReference)
      : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StopCellRequest>, I>>(object: I): StopCellRequest {
    const message = createBaseStopCellRequest();
    message.executableReference = (object.executableReference !== undefined && object.executableReference !== null)
      ? ExecutableReference.fromPartial(object.executableReference)
      : undefined;
    return message;
  },
};

function createBaseStopCellResponse(): StopCellResponse {
  return {};
}

export const StopCellResponse = {
  fromJSON(_: any): StopCellResponse {
    return {};
  },

  toJSON(_: StopCellResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StopCellResponse>, I>>(_: I): StopCellResponse {
    const message = createBaseStopCellResponse();
    return message;
  },
};

/** TODO Pods Service */
export interface Pods {
}

/** TODO Instances Service */
export interface Instances {
}

/** TODO Spawn Service */
export interface Spawn {
}

/**
 * / Cells is the most fundamental isolation boundary for Aurae.
 * / A cell is an isolate set of resources of the system which can be
 * / used to run workloads.
 * /
 * / A cell is composed of a unique cgroup namespace, and unshared kernel namespaces.
 */
export interface CellService {
  /**
   * / Reserve requested system resources for a new cell.
   * / For cells specifically this will allocate and reserve cgroup resources only.
   */
  allocate(request: AllocateCellRequest): Promise<AllocateCellResponse>;
  /** / Free up previously requested resources for an existing cell */
  free(request: FreeCellRequest): Promise<FreeCellResponse>;
  /**
   * / Start a new Executable inside of an existing cell. Can be called
   * / in serial to start more than one executable in the same cell.
   */
  start(request: StartCellRequest): Promise<StartCellResponse>;
  /**
   * / Stop one or more Executables inside of an existing cell.
   * / Can be called in serial to stop/retry more than one executable.
   */
  stop(request: StopCellRequest): Promise<StopCellResponse>;
}

type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;

export type DeepPartial<T> = T extends Builtin ? T
  : T extends Array<infer U> ? Array<DeepPartial<U>> : T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>>
  : T extends {} ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type Exact<P, I extends P> = P extends Builtin ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & { [K in Exclude<keyof I, KeysOfUnion<P>>]: never };

function isSet(value: any): boolean {
  return value !== null && value !== undefined;
}
export class CellServiceClient implements CellService {
allocate(request: AllocateCellRequest): Promise<AllocateCellResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__cell_service__allocate(request);
}      
        
free(request: FreeCellRequest): Promise<FreeCellResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__cell_service__free(request);
}      
        
start(request: StartCellRequest): Promise<StartCellResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__cell_service__start(request);
}      
        
stop(request: StopCellRequest): Promise<StopCellResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__cell_service__stop(request);
}      
        }