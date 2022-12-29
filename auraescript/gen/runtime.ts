/* eslint-disable */

export const protobufPackage = "runtime";

export interface AllocatePodRequest {
}

export interface AllocatePodResponse {
}

export interface FreePodRequest {
}

export interface FreePodResponse {
}

export interface StartContainerRequest {
}

export interface StartContainerResponse {
}

export interface StopContainerRequest {
}

export interface StopContainerResponse {
}

/** / The most primitive workload in Aurae, a standard executable process. */
export interface Executable {
  name: string;
  command: string;
  description: string;
}

/**
 * / An isolation resource used to divide a system into smaller resource
 * / boundaries.
 */
export interface Cell {
  /**
   * / Resource parameters for control groups (cgroups)
   * / Build on the [cgroups-rs](https://github.com/kata-containers/cgroups-rs)
   * / crate. See
   * / [examples](https://github.com/kata-containers/cgroups-rs/blob/main/tests/builder.rs)
   */
  name: string;
  /**
   * / A comma-separated list of CPU IDs where the task in the control group
   * / can run. Dashes between numbers indicate ranges.
   */
  cpuCpus: string;
  /**
   * /  Cgroups can be guaranteed a minimum number of "CPU shares"
   * /  when a system is busy.  This does not limit a cgroup's CPU
   * /  usage if the CPUs are not busy.  For further information,
   * /  see Documentation/scheduler/sched-design-CFS.rst (or
   * /  Documentation/scheduler/sched-design-CFS.txt in Linux 5.2
   * /  and earlier).
   * /
   * / Weight of how much of the total CPU time should this control
   * /  group get. Note that this is hierarchical, so this is weighted
   * /  against the siblings of this control group.
   */
  cpuShares: number;
  /**
   * / Same syntax as the cpus field of this structure, but applies to
   * /  memory nodes instead of processors.
   */
  cpuMems: string;
  /** / In one period, how much can the tasks run in microseconds. */
  cpuQuota: number;
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

/**
 * / An Aurae cell is a name given to Linux control groups (cgroups) that also include
 * / a name, and special pre-exec functionality that is executed from within the same context
 * / as any executables scheduled.
 * /
 * / A cell must be allocated for every executable scheduled. A cell defines the resource
 * / constraints of the system to allocate for an arbitrary use case.
 */
export interface AllocateCellRequest {
  /** / A smaller resource constrained section of the system. */
  cell: Cell | undefined;
}

/** / The response after a cell has been allocated. */
export interface AllocateCellResponse {
  cellName: string;
  /**
   * / A bool that will be set to true if the cgroup was created with
   * / cgroup v2 controller.
   */
  cgroupV2: boolean;
}

/** / Used to remove or free a cell after it has been allocated. */
export interface FreeCellRequest {
  cellName: string;
}

/** / Response after removing or freeing a cell. */
export interface FreeCellResponse {
}

/**
 * / A request for starting an executable inside of a Cell.
 * /
 * / This is the lowest level of raw executive functionality.
 * / Here you can define shell commands, and meta information about the command.
 * / An executable is started synchronously.
 */
export interface StartExecutableRequest {
  cellName: string;
  executable: Executable | undefined;
}

/** / The response after starting an executable within a Cell. */
export interface StartExecutableResponse {
  /**
   * / Return a pid as an int32 based on the pid_t type
   * / in various libc libraries.
   */
  pid: number;
}

/** / Request to stop an executable at runtime. */
export interface StopExecutableRequest {
  cellName: string;
  executableName: string;
}

export interface StopExecutableResponse {
}

function createBaseAllocatePodRequest(): AllocatePodRequest {
  return {};
}

export const AllocatePodRequest = {
  fromJSON(_: any): AllocatePodRequest {
    return {};
  },

  toJSON(_: AllocatePodRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<AllocatePodRequest>, I>>(_: I): AllocatePodRequest {
    const message = createBaseAllocatePodRequest();
    return message;
  },
};

function createBaseAllocatePodResponse(): AllocatePodResponse {
  return {};
}

export const AllocatePodResponse = {
  fromJSON(_: any): AllocatePodResponse {
    return {};
  },

  toJSON(_: AllocatePodResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<AllocatePodResponse>, I>>(_: I): AllocatePodResponse {
    const message = createBaseAllocatePodResponse();
    return message;
  },
};

function createBaseFreePodRequest(): FreePodRequest {
  return {};
}

export const FreePodRequest = {
  fromJSON(_: any): FreePodRequest {
    return {};
  },

  toJSON(_: FreePodRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FreePodRequest>, I>>(_: I): FreePodRequest {
    const message = createBaseFreePodRequest();
    return message;
  },
};

function createBaseFreePodResponse(): FreePodResponse {
  return {};
}

export const FreePodResponse = {
  fromJSON(_: any): FreePodResponse {
    return {};
  },

  toJSON(_: FreePodResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FreePodResponse>, I>>(_: I): FreePodResponse {
    const message = createBaseFreePodResponse();
    return message;
  },
};

function createBaseStartContainerRequest(): StartContainerRequest {
  return {};
}

export const StartContainerRequest = {
  fromJSON(_: any): StartContainerRequest {
    return {};
  },

  toJSON(_: StartContainerRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StartContainerRequest>, I>>(_: I): StartContainerRequest {
    const message = createBaseStartContainerRequest();
    return message;
  },
};

function createBaseStartContainerResponse(): StartContainerResponse {
  return {};
}

export const StartContainerResponse = {
  fromJSON(_: any): StartContainerResponse {
    return {};
  },

  toJSON(_: StartContainerResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StartContainerResponse>, I>>(_: I): StartContainerResponse {
    const message = createBaseStartContainerResponse();
    return message;
  },
};

function createBaseStopContainerRequest(): StopContainerRequest {
  return {};
}

export const StopContainerRequest = {
  fromJSON(_: any): StopContainerRequest {
    return {};
  },

  toJSON(_: StopContainerRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StopContainerRequest>, I>>(_: I): StopContainerRequest {
    const message = createBaseStopContainerRequest();
    return message;
  },
};

function createBaseStopContainerResponse(): StopContainerResponse {
  return {};
}

export const StopContainerResponse = {
  fromJSON(_: any): StopContainerResponse {
    return {};
  },

  toJSON(_: StopContainerResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StopContainerResponse>, I>>(_: I): StopContainerResponse {
    const message = createBaseStopContainerResponse();
    return message;
  },
};

function createBaseExecutable(): Executable {
  return { name: "", command: "", description: "" };
}

export const Executable = {
  fromJSON(object: any): Executable {
    return {
      name: isSet(object.name) ? String(object.name) : "",
      command: isSet(object.command) ? String(object.command) : "",
      description: isSet(object.description) ? String(object.description) : "",
    };
  },

  toJSON(message: Executable): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.command !== undefined && (obj.command = message.command);
    message.description !== undefined && (obj.description = message.description);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Executable>, I>>(object: I): Executable {
    const message = createBaseExecutable();
    message.name = object.name ?? "";
    message.command = object.command ?? "";
    message.description = object.description ?? "";
    return message;
  },
};

function createBaseCell(): Cell {
  return {
    name: "",
    cpuCpus: "",
    cpuShares: 0,
    cpuMems: "",
    cpuQuota: 0,
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
      cpuCpus: isSet(object.cpuCpus) ? String(object.cpuCpus) : "",
      cpuShares: isSet(object.cpuShares) ? Number(object.cpuShares) : 0,
      cpuMems: isSet(object.cpuMems) ? String(object.cpuMems) : "",
      cpuQuota: isSet(object.cpuQuota) ? Number(object.cpuQuota) : 0,
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
    message.cpuCpus !== undefined && (obj.cpuCpus = message.cpuCpus);
    message.cpuShares !== undefined && (obj.cpuShares = Math.round(message.cpuShares));
    message.cpuMems !== undefined && (obj.cpuMems = message.cpuMems);
    message.cpuQuota !== undefined && (obj.cpuQuota = Math.round(message.cpuQuota));
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
    message.cpuCpus = object.cpuCpus ?? "";
    message.cpuShares = object.cpuShares ?? 0;
    message.cpuMems = object.cpuMems ?? "";
    message.cpuQuota = object.cpuQuota ?? 0;
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
  return { cellName: "", cgroupV2: false };
}

export const AllocateCellResponse = {
  fromJSON(object: any): AllocateCellResponse {
    return {
      cellName: isSet(object.cellName) ? String(object.cellName) : "",
      cgroupV2: isSet(object.cgroupV2) ? Boolean(object.cgroupV2) : false,
    };
  },

  toJSON(message: AllocateCellResponse): unknown {
    const obj: any = {};
    message.cellName !== undefined && (obj.cellName = message.cellName);
    message.cgroupV2 !== undefined && (obj.cgroupV2 = message.cgroupV2);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<AllocateCellResponse>, I>>(object: I): AllocateCellResponse {
    const message = createBaseAllocateCellResponse();
    message.cellName = object.cellName ?? "";
    message.cgroupV2 = object.cgroupV2 ?? false;
    return message;
  },
};

function createBaseFreeCellRequest(): FreeCellRequest {
  return { cellName: "" };
}

export const FreeCellRequest = {
  fromJSON(object: any): FreeCellRequest {
    return { cellName: isSet(object.cellName) ? String(object.cellName) : "" };
  },

  toJSON(message: FreeCellRequest): unknown {
    const obj: any = {};
    message.cellName !== undefined && (obj.cellName = message.cellName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FreeCellRequest>, I>>(object: I): FreeCellRequest {
    const message = createBaseFreeCellRequest();
    message.cellName = object.cellName ?? "";
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

function createBaseStartExecutableRequest(): StartExecutableRequest {
  return { cellName: "", executable: undefined };
}

export const StartExecutableRequest = {
  fromJSON(object: any): StartExecutableRequest {
    return {
      cellName: isSet(object.cellName) ? String(object.cellName) : "",
      executable: isSet(object.executable) ? Executable.fromJSON(object.executable) : undefined,
    };
  },

  toJSON(message: StartExecutableRequest): unknown {
    const obj: any = {};
    message.cellName !== undefined && (obj.cellName = message.cellName);
    message.executable !== undefined &&
      (obj.executable = message.executable ? Executable.toJSON(message.executable) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StartExecutableRequest>, I>>(object: I): StartExecutableRequest {
    const message = createBaseStartExecutableRequest();
    message.cellName = object.cellName ?? "";
    message.executable = (object.executable !== undefined && object.executable !== null)
      ? Executable.fromPartial(object.executable)
      : undefined;
    return message;
  },
};

function createBaseStartExecutableResponse(): StartExecutableResponse {
  return { pid: 0 };
}

export const StartExecutableResponse = {
  fromJSON(object: any): StartExecutableResponse {
    return { pid: isSet(object.pid) ? Number(object.pid) : 0 };
  },

  toJSON(message: StartExecutableResponse): unknown {
    const obj: any = {};
    message.pid !== undefined && (obj.pid = Math.round(message.pid));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StartExecutableResponse>, I>>(object: I): StartExecutableResponse {
    const message = createBaseStartExecutableResponse();
    message.pid = object.pid ?? 0;
    return message;
  },
};

function createBaseStopExecutableRequest(): StopExecutableRequest {
  return { cellName: "", executableName: "" };
}

export const StopExecutableRequest = {
  fromJSON(object: any): StopExecutableRequest {
    return {
      cellName: isSet(object.cellName) ? String(object.cellName) : "",
      executableName: isSet(object.executableName) ? String(object.executableName) : "",
    };
  },

  toJSON(message: StopExecutableRequest): unknown {
    const obj: any = {};
    message.cellName !== undefined && (obj.cellName = message.cellName);
    message.executableName !== undefined && (obj.executableName = message.executableName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StopExecutableRequest>, I>>(object: I): StopExecutableRequest {
    const message = createBaseStopExecutableRequest();
    message.cellName = object.cellName ?? "";
    message.executableName = object.executableName ?? "";
    return message;
  },
};

function createBaseStopExecutableResponse(): StopExecutableResponse {
  return {};
}

export const StopExecutableResponse = {
  fromJSON(_: any): StopExecutableResponse {
    return {};
  },

  toJSON(_: StopExecutableResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<StopExecutableResponse>, I>>(_: I): StopExecutableResponse {
    const message = createBaseStopExecutableResponse();
    return message;
  },
};

/** TODO Instances Service */
export interface Instances {
}

/** TODO Spawn Service */
export interface Spawn {
}

/**
 * / A pod is a higher level abstraction than Aurae cells, and to most users
 * / will look at feel like one or more "containers".
 * /
 * / Pods will run an OCI compliant container image.
 * /
 * / A pod is a group of one or more containers with shared network and storage.
 */
export interface PodService {
  allocate(request: AllocatePodRequest): Promise<AllocatePodResponse>;
  start(request: StartContainerRequest): Promise<StartContainerResponse>;
  stop(request: StopContainerRequest): Promise<StopContainerResponse>;
  free(request: FreePodRequest): Promise<FreePodResponse>;
}

/**
 * / Cells is the most fundamental isolation boundary for Aurae.
 * / A cell is an isolate set of resources of the system which can be
 * / used to run workloads.
 * /
 * / A cell is composed of a unique cgroup namespace, and unshared kernel
 * / namespaces.
 */
export interface CellService {
  /**
   * / Reserve requested system resources for a new cell.
   * / For cells specifically this will allocate and reserve cgroup resources
   * / only.
   */
  allocate(request: AllocateCellRequest): Promise<AllocateCellResponse>;
  /** / Free up previously requested resources for an existing cell */
  free(request: FreeCellRequest): Promise<FreeCellResponse>;
  /**
   * / Start a new Executable inside of an existing cell. Can be called
   * / in serial to start more than one executable in the same cell.
   */
  start(request: StartExecutableRequest): Promise<StartExecutableResponse>;
  /**
   * / Stop one or more Executables inside of an existing cell.
   * / Can be called in serial to stop/retry more than one executable.
   */
  stop(request: StopExecutableRequest): Promise<StopExecutableResponse>;
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
        
start(request: StartExecutableRequest): Promise<StartExecutableResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__cell_service__start(request);
}      
        
stop(request: StopExecutableRequest): Promise<StopExecutableResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__cell_service__stop(request);
}      
        }

export class PodServiceClient implements PodService {
allocate(request: AllocatePodRequest): Promise<AllocatePodResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__pod_service__allocate(request);
}      
        
free(request: FreePodRequest): Promise<FreePodResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__pod_service__free(request);
}      
        
start(request: StartContainerRequest): Promise<StartContainerResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__pod_service__start(request);
}      
        
stop(request: StopContainerRequest): Promise<StopContainerResponse> {
    // @ts-ignore
    return Deno.core.ops.ae__runtime__pod_service__stop(request);
}      
        }