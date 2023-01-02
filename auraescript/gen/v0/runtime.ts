/* eslint-disable */

export const protobufPackage = "aurae.runtime.v0";

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
export interface CellServiceAllocateRequest {
  /** / A smaller resource constrained section of the system. */
  cell: Cell | undefined;
}

/** / The response after a cell has been allocated. */
export interface CellServiceAllocateResponse {
  cellName: string;
  /**
   * / A bool that will be set to true if the cgroup was created with
   * / cgroup v2 controller.
   */
  cgroupV2: boolean;
}

/** / Used to remove or free a cell after it has been allocated. */
export interface CellServiceFreeRequest {
  cellName: string;
}

/** / Response after removing or freeing a cell. */
export interface CellServiceFreeResponse {
}

/**
 * / A request for starting an executable inside of a Cell.
 * /
 * / This is the lowest level of raw executive functionality.
 * / Here you can define shell commands, and meta information about the command.
 * / An executable is started synchronously.
 */
export interface CellServiceStartRequest {
  cellName: string;
  executable: Executable | undefined;
}

/** / The response after starting an executable within a Cell. */
export interface CellServiceStartResponse {
  /**
   * / Return a pid as an int32 based on the pid_t type
   * / in various libc libraries.
   */
  pid: number;
}

/** / Request to stop an executable at runtime. */
export interface CellServiceStopRequest {
  cellName: string;
  executableName: string;
}

export interface CellServiceStopResponse {
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

function createBaseCellServiceAllocateRequest(): CellServiceAllocateRequest {
  return { cell: undefined };
}

export const CellServiceAllocateRequest = {
  fromJSON(object: any): CellServiceAllocateRequest {
    return { cell: isSet(object.cell) ? Cell.fromJSON(object.cell) : undefined };
  },

  toJSON(message: CellServiceAllocateRequest): unknown {
    const obj: any = {};
    message.cell !== undefined && (obj.cell = message.cell ? Cell.toJSON(message.cell) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CellServiceAllocateRequest>, I>>(object: I): CellServiceAllocateRequest {
    const message = createBaseCellServiceAllocateRequest();
    message.cell = (object.cell !== undefined && object.cell !== null) ? Cell.fromPartial(object.cell) : undefined;
    return message;
  },
};

function createBaseCellServiceAllocateResponse(): CellServiceAllocateResponse {
  return { cellName: "", cgroupV2: false };
}

export const CellServiceAllocateResponse = {
  fromJSON(object: any): CellServiceAllocateResponse {
    return {
      cellName: isSet(object.cellName) ? String(object.cellName) : "",
      cgroupV2: isSet(object.cgroupV2) ? Boolean(object.cgroupV2) : false,
    };
  },

  toJSON(message: CellServiceAllocateResponse): unknown {
    const obj: any = {};
    message.cellName !== undefined && (obj.cellName = message.cellName);
    message.cgroupV2 !== undefined && (obj.cgroupV2 = message.cgroupV2);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CellServiceAllocateResponse>, I>>(object: I): CellServiceAllocateResponse {
    const message = createBaseCellServiceAllocateResponse();
    message.cellName = object.cellName ?? "";
    message.cgroupV2 = object.cgroupV2 ?? false;
    return message;
  },
};

function createBaseCellServiceFreeRequest(): CellServiceFreeRequest {
  return { cellName: "" };
}

export const CellServiceFreeRequest = {
  fromJSON(object: any): CellServiceFreeRequest {
    return { cellName: isSet(object.cellName) ? String(object.cellName) : "" };
  },

  toJSON(message: CellServiceFreeRequest): unknown {
    const obj: any = {};
    message.cellName !== undefined && (obj.cellName = message.cellName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CellServiceFreeRequest>, I>>(object: I): CellServiceFreeRequest {
    const message = createBaseCellServiceFreeRequest();
    message.cellName = object.cellName ?? "";
    return message;
  },
};

function createBaseCellServiceFreeResponse(): CellServiceFreeResponse {
  return {};
}

export const CellServiceFreeResponse = {
  fromJSON(_: any): CellServiceFreeResponse {
    return {};
  },

  toJSON(_: CellServiceFreeResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CellServiceFreeResponse>, I>>(_: I): CellServiceFreeResponse {
    const message = createBaseCellServiceFreeResponse();
    return message;
  },
};

function createBaseCellServiceStartRequest(): CellServiceStartRequest {
  return { cellName: "", executable: undefined };
}

export const CellServiceStartRequest = {
  fromJSON(object: any): CellServiceStartRequest {
    return {
      cellName: isSet(object.cellName) ? String(object.cellName) : "",
      executable: isSet(object.executable) ? Executable.fromJSON(object.executable) : undefined,
    };
  },

  toJSON(message: CellServiceStartRequest): unknown {
    const obj: any = {};
    message.cellName !== undefined && (obj.cellName = message.cellName);
    message.executable !== undefined &&
      (obj.executable = message.executable ? Executable.toJSON(message.executable) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CellServiceStartRequest>, I>>(object: I): CellServiceStartRequest {
    const message = createBaseCellServiceStartRequest();
    message.cellName = object.cellName ?? "";
    message.executable = (object.executable !== undefined && object.executable !== null)
      ? Executable.fromPartial(object.executable)
      : undefined;
    return message;
  },
};

function createBaseCellServiceStartResponse(): CellServiceStartResponse {
  return { pid: 0 };
}

export const CellServiceStartResponse = {
  fromJSON(object: any): CellServiceStartResponse {
    return { pid: isSet(object.pid) ? Number(object.pid) : 0 };
  },

  toJSON(message: CellServiceStartResponse): unknown {
    const obj: any = {};
    message.pid !== undefined && (obj.pid = Math.round(message.pid));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CellServiceStartResponse>, I>>(object: I): CellServiceStartResponse {
    const message = createBaseCellServiceStartResponse();
    message.pid = object.pid ?? 0;
    return message;
  },
};

function createBaseCellServiceStopRequest(): CellServiceStopRequest {
  return { cellName: "", executableName: "" };
}

export const CellServiceStopRequest = {
  fromJSON(object: any): CellServiceStopRequest {
    return {
      cellName: isSet(object.cellName) ? String(object.cellName) : "",
      executableName: isSet(object.executableName) ? String(object.executableName) : "",
    };
  },

  toJSON(message: CellServiceStopRequest): unknown {
    const obj: any = {};
    message.cellName !== undefined && (obj.cellName = message.cellName);
    message.executableName !== undefined && (obj.executableName = message.executableName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CellServiceStopRequest>, I>>(object: I): CellServiceStopRequest {
    const message = createBaseCellServiceStopRequest();
    message.cellName = object.cellName ?? "";
    message.executableName = object.executableName ?? "";
    return message;
  },
};

function createBaseCellServiceStopResponse(): CellServiceStopResponse {
  return {};
}

export const CellServiceStopResponse = {
  fromJSON(_: any): CellServiceStopResponse {
    return {};
  },

  toJSON(_: CellServiceStopResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CellServiceStopResponse>, I>>(_: I): CellServiceStopResponse {
    const message = createBaseCellServiceStopResponse();
    return message;
  },
};

/** TODO Instance Service */
export interface InstanceService {
}

/** TODO Spawn Service */
export interface SpawnService {
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
  allocate(request: PodServiceAllocateRequest): Promise<PodServiceAllocateResponse>;
  start(request: PodServiceStartRequest): Promise<PodServiceStartResponse>;
  stop(request: PodServiceStopRequest): Promise<PodServiceStopResponse>;
  free(request: PodServiceFreeRequest): Promise<PodServiceFreeResponse>;
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
  allocate(request: CellServiceAllocateRequest): Promise<CellServiceAllocateResponse>;
  /** / Free up previously requested resources for an existing cell */
  free(request: CellServiceFreeRequest): Promise<CellServiceFreeResponse>;
  /**
   * / Start a new Executable inside of an existing cell. Can be called
   * / in serial to start more than one executable in the same cell.
   */
  start(request: CellServiceStartRequest): Promise<CellServiceStartResponse>;
  /**
   * / Stop one or more Executables inside of an existing cell.
   * / Can be called in serial to stop/retry more than one executable.
   */
  stop(request: CellServiceStopRequest): Promise<CellServiceStopResponse>;
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
