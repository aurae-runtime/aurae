# Signals

The Aurae project calls out general rules for how all daemons (including `auraed` itself) should respond to various POSIX signals. 

The `auraed` daemon will proxy signals sent to nested cells and nested `auraed` instances.


| Signal  | Value | Proxy   | Description                                                                                                                           |
|---------|-------|---------|---------------------------------------------------------------------------------------------------------------------------------------|
| SIGKILL | 9     | SIGKILL | The most destructive signal. Will immediately kill `auraed`.                                                                          |
| SIGHUP  | 1     | SIGHUP  | Sent when a controlling shell, or TTY is closed. Used to reload `auraed` and reopen file descriptors.                                 |
| SIGTERM | 15    | SIGTERM | Used to tell a nested `auraed` it is time to "die nicely" and begin stopping workloads in the cache, and destroying nested resources. |
| SIGINT  | 2     | SIGINT  | Ignored by `auraed`                                                                                                                   |


## Observe signals with auraed eBPF

```bash 
aer observe get-posix-signals-stream
```


