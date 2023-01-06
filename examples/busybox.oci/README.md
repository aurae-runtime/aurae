# Busybox Destructed

I went ahead and took apart the OCI container image so we can edit and view the files directly.

 - busybox.tar.gz is the actual "tarball" that is the container "image"
 - the contents of this directory are an export of the "docker" version of this container
 - the contents of the ./busybox directory are "the container filesystem" or "bundle" directory
