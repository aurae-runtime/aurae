# Developing on macOS with Lima

This development guide is intended for any macOS system with any text editor or IDE.

Lima is a virtual machine launcher, mainly built for macOS, that provides convenient mechanisms for interacting and configuring virtual machines out of the box.

## Setup
1. Install lima. You can find the instructions at [Lima's Getting Started](https://github.com/lima-vm/lima#getting-started) documentation

2. Create an Ubuntu virtual machine. This command will walk you through the installation process:
   ```sh
   $ limactl start --name aurae
   ```

3. SSH into your virtual machine
   ```sh
   $ limactl shell aurae
   ```

4. Follow the aurae [Building from Source](https://aurae.io/build/) instructions. You can edit either stop the virtual
   machine, edit its configuration to create a writable mount of the source code directory from your host machine to the
   guest, or copy the contents of the source code directory to `/tmp/lima`, e.g., `/tmp/lima/aurae`

5. **[Optional]** Mount the unix socket from the guest machine to the host machine. Lima provides a simple way to
   configure this, but you first need to stop your virtual machine to edit its configuration:
   ```sh
   $ limactl stop aurea
   ```
   Now you can edit the config:
   ```sh
   $ limactl edit aurea
   ```
   And you can add a configuration like this to the bottom:
   ```yaml
   portForwards:
   - guestSocket: "/var/run/aurae/aurae.sock"
     hostSocket: "aurae.sock"
   ```
   Then start your virtual machine again and start auraed:
   ```sh
   $ limactl start aurae
   $ limactl shell make auraed-start
   ```
   In another shell copy the virtual machine's `~/.aurae` directory to your own:
   ```sh
   $ limactl copy -r aurae:~/.aurae ~/.aurae
   ```
   Edit the contents of `~/.aurae/config` to fit the requirements of your host machine. Now you can interact with aurae
   from your host machine!

6. **[Optional]** If you're tired of specifying the name you gave your virtual machine, you can export an environment
   variable that `limactl` will use for the default, like so:
   ```sh
   $ export LIMA_INSTANCE=aurae
   ```
   
