/* eslint-disable */
import { Observable } from "rxjs";

export const protobufPackage = "aurae.observe.v0";

export enum LogChannelType {
  LOG_CHANNEL_TYPE_UNSPECIFIED = 0,
  LOG_CHANNEL_TYPE_STDOUT = 1,
  LOG_CHANNEL_TYPE_STDERR = 2,
  UNRECOGNIZED = -1,
}

export function logChannelTypeFromJSON(object: any): LogChannelType {
  switch (object) {
    case 0:
    case "LOG_CHANNEL_TYPE_UNSPECIFIED":
      return LogChannelType.LOG_CHANNEL_TYPE_UNSPECIFIED;
    case 1:
    case "LOG_CHANNEL_TYPE_STDOUT":
      return LogChannelType.LOG_CHANNEL_TYPE_STDOUT;
    case 2:
    case "LOG_CHANNEL_TYPE_STDERR":
      return LogChannelType.LOG_CHANNEL_TYPE_STDERR;
    case -1:
    case "UNRECOGNIZED":
    default:
      return LogChannelType.UNRECOGNIZED;
  }
}

export function logChannelTypeToJSON(object: LogChannelType): string {
  switch (object) {
    case LogChannelType.LOG_CHANNEL_TYPE_UNSPECIFIED:
      return "LOG_CHANNEL_TYPE_UNSPECIFIED";
    case LogChannelType.LOG_CHANNEL_TYPE_STDOUT:
      return "LOG_CHANNEL_TYPE_STDOUT";
    case LogChannelType.LOG_CHANNEL_TYPE_STDERR:
      return "LOG_CHANNEL_TYPE_STDERR";
    case LogChannelType.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

export interface GetAuraeDaemonLogStreamRequest {
}

/** TODO: not implemented */
export interface GetSubProcessStreamRequest {
  channelType: LogChannelType;
  processId: number;
}

export interface LogItem {
  channel: string;
  line: string;
  timestamp: number;
}

function createBaseGetAuraeDaemonLogStreamRequest(): GetAuraeDaemonLogStreamRequest {
  return {};
}

export const GetAuraeDaemonLogStreamRequest = {
  fromJSON(_: any): GetAuraeDaemonLogStreamRequest {
    return {};
  },

  toJSON(_: GetAuraeDaemonLogStreamRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<GetAuraeDaemonLogStreamRequest>, I>>(_: I): GetAuraeDaemonLogStreamRequest {
    const message = createBaseGetAuraeDaemonLogStreamRequest();
    return message;
  },
};

function createBaseGetSubProcessStreamRequest(): GetSubProcessStreamRequest {
  return { channelType: 0, processId: 0 };
}

export const GetSubProcessStreamRequest = {
  fromJSON(object: any): GetSubProcessStreamRequest {
    return {
      channelType: isSet(object.channelType) ? logChannelTypeFromJSON(object.channelType) : 0,
      processId: isSet(object.processId) ? Number(object.processId) : 0,
    };
  },

  toJSON(message: GetSubProcessStreamRequest): unknown {
    const obj: any = {};
    message.channelType !== undefined && (obj.channelType = logChannelTypeToJSON(message.channelType));
    message.processId !== undefined && (obj.processId = Math.round(message.processId));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<GetSubProcessStreamRequest>, I>>(object: I): GetSubProcessStreamRequest {
    const message = createBaseGetSubProcessStreamRequest();
    message.channelType = object.channelType ?? 0;
    message.processId = object.processId ?? 0;
    return message;
  },
};

function createBaseLogItem(): LogItem {
  return { channel: "", line: "", timestamp: 0 };
}

export const LogItem = {
  fromJSON(object: any): LogItem {
    return {
      channel: isSet(object.channel) ? String(object.channel) : "",
      line: isSet(object.line) ? String(object.line) : "",
      timestamp: isSet(object.timestamp) ? Number(object.timestamp) : 0,
    };
  },

  toJSON(message: LogItem): unknown {
    const obj: any = {};
    message.channel !== undefined && (obj.channel = message.channel);
    message.line !== undefined && (obj.line = message.line);
    message.timestamp !== undefined && (obj.timestamp = Math.round(message.timestamp));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<LogItem>, I>>(object: I): LogItem {
    const message = createBaseLogItem();
    message.channel = object.channel ?? "";
    message.line = object.line ?? "";
    message.timestamp = object.timestamp ?? 0;
    return message;
  },
};

export interface ObserveService {
  /** request log stream for aurae. everything logged via log macros in aurae (info!, error!, trace!, ... ). */
  getAuraeDaemonLogStream(request: GetAuraeDaemonLogStreamRequest): Observable<LogItem>;
  /** TODO: request log stream for a sub process */
  getSubProcessStream(request: GetSubProcessStreamRequest): Observable<LogItem>;
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
