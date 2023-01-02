# Runtime

The Runtime API is a resource-oriented set of services. Resource type API modelled after the "Resource-oriented design" shared by [Source](https://google.aip.dev/121).

## Runtime Verbs

We call out a set of common verbs to the runtime subsystem. These verbs should be generic enough to use for each resource added to the RPCs and services.

If functionality can not be implemented by one of these verbs a new verb may be introduced as long as it reasonably applicable to similar RPCs and services.

* Allocate
  * Reserve resources, and manage any prerequisites but do not start
* Free
  * Free resources, and destroy any prerequisites that have been started
* Start
  * Run a resource immediately
* Stop
  * Stop a resource immediately
* Spawn
  * A special function that creates a child instance with inherited properties of the parent
