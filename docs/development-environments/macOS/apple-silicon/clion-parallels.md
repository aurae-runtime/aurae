# Developing on Apple silicon with CLion and Parallels Desktop

This is a development environment setup guide that may be helpful for those on Apple silicon. There are likely
improvements to be made, but the steps are purposefully overly "hand hold-y" and detailed to be useful to a wider
audience.

This guide assumes you are following it from top to bottom.

## Environment

*The environment used when writing this guide:*

- **Processor**: Apple M1
- **operating system**: macOS Ventura
- **Virtualizer** one of:
    - Parallels Desktop for Mac 18 Pro Edition
        - *A Standard edition exists, and may be enough. Parallels offers a trial period.*
    - UTM emulator/virtualizer
        - *Free, though its performance is not guaranteed and may take more work for you*  
- **Virtual Machine (VM) operating system**: Ubuntu 22.04 ARM64 (available in the choice list when creating a VM in
  Parallels)
- **IDE**: JetBrains CLion 2022.3
    - *Visual Studio Code will likely work as well, but the steps for setup are not noted in this guide as of yet.*

## Parallels VM Setup

1. Download and install Parallels Desktop for Mac
2. Create a new VM (File -> New -> Download Ubuntu Linux -> Continue)
3. After the VM is installed and starts up, shut down the VM before logging in
4. Open Parallels Control Center (Window -> Control Center)
5. Click the gear icon next to the VM to open the VM's Configuration menu to make the configuration to your preferences.
   The following are mine:
    - Options -> Sharing -> Disable all but "Share Mac volumes with Linux"
    - Options -> Sharing -> Disable "Share Linux applications with Mac"
    - Hardware -> Processors -> 6 (Standard edition allows up to 4, which should be fine)
    - Hardware -> Memory -> 16384 MB (16GB; Standard edition allows up to 8GB which should be fine)
    - Graphics -> More Space (scaling will be done in the VM)
6. Start the VM again and enter a password to set it. You may have some windows demanding your attention:
    - Parallels asking for the password to use `sudo` and install Parallels Tools. Enter your password. Once it is done
      installing, it will start a countdown to restart the VM...click postpone, or be quick.
    - Livepatch setup, which seems impossible to just close.
        - Continue -> "No, don't send system info" (unless you want to) -> Next -> Next -> Done
7. Restart and log in again
8. (Optional) Right click the desktop -> Display Settings -> Adjust to your preferences
9. (Optional, still in Settings) Keyboard -> View and Customize Shortcuts -> Search for "lock" -> Disable it so you
   don't accidentally lock your VM every time you want to clear the terminal.
10. (Optional, still in Settings) Power -> Screen Blank -> Never
11. (Optional) Right click the desktop -> Open in Terminal -> Click the Hamburger menu button -> Adjust to your
    preferences. The following are mine:
    - General -> Theme variant -> Dark
    - Unnamed -> Colors -> Text and Background Color -> Set Built-in schemes = Solarized dark
    - Unnamed -> Colors -> Palette -> Set Built-in schemes = Solarized
12. Create an empty directory where the Aurae files will be synced. This guide assumes you will create it on the desktop
    at `Desktop/aurae`.
13. (Optional) Now might be a good time to snapshot the VM (feel free to do it at any other step)
    1. Open Parallels Control Center (Window -> Control Center)
    2. Right click your VM -> Manage Snapshots -> New -> Set a name ("Init") -> Ok

## UTM Setup

**Do not do this if you're using Parallels. You only need one VM.**

1. Download and install UTM from releases https://github.com/utmapp/UTM/releases
2. Download the Ubuntu ISO and create a new VM: https://docs.getutm.app/guides/ubuntu/
3. I never got clipboard sharing working, just decided to SSH in and rely on my host system. `ip a | grep addr` to get the address and then regular `ssh username@ip` (and whatever extras you prefer to set)
4. Set up shared drive: follow https://docs.getutm.app/guest-support/linux/#virtfs then
    - add this line to `/etc/fstab`:
        ```
        share	[mount point]	9p	trans=virtio,version=9p2000.L,rw,_netdev,nofail	0	0
        ```
    -  Mount share and fix permissions (does not modify the host files, just inside-VM settings. Preserved between restarts.):
        ```
        sudo mount /share
        sudo chown 1000:1000 /share
        ```

Everything else should proceed the same.

## CLion Setup

1. Download and install CLion.
    - *This is on your mac, not in the VM. We will use CLion's remote development features to connect to the VM.*
2. Open the Aurae project in CLion
3. Install plugins (CLion -> Setting -> Plugins)
    - Rust
    - Protocol Buffers
    - Deno
4. Restart the IDE to finish installing the plugins
5. Activate the experimental Rust plugin features
    1. Open CLion's "search everywhere" window (can be opened via magnifying glass in upper right corner)
    2. Search for and select "Experimental Features"
    3. Activate all options that start with "org.rust"
6. Setup remote development (CLion -> Settings -> Build, Execution, Deployment -> Development)
    1. Click the + icon -> SFTP -> Enter a name ("AuraeVM")
    2. Connection -> SSH configuration -> click the "..."
        1. Set Host to the IP address of the VM (Parallels -> Devices -> Network -> an IP address should be listed)
        2. Set Username to "parallels"
        3. Set Password to your VM's password -> select "Save Password" -> de-select "Parse config file ~/.ssh/config"
        4. Clicking Test Connection should show a success message
        5. Select Ok, to close the menu (we will be back in Settings)
    3. Connection -> Root path -> click Autodetect (my root path becomes "/home/parallels")
    4. Connection -> Select "Use Rsync for download/upload/sync"
    5. Mappings -> Local path -> should be set to the path to the Aurae project on your mac
    6. Mappings -> Deployment path -> `Desktop/aurae` (the empty directory created in the VM)
    7. Excluded Paths -> + -> Local path -> path to the "target" directory in the Aurae project directory on your mac
    8. Click apply to save the settings so far
7. Configure development options (CLion -> Settings -> Build, Execution, Deployment -> Development -> Options)
    - The following options are selected:
        - Overwrite up-to-date files
        - Preserve file timestamps
        - Delete target items when source ones do not exist
        - Create empty directories
        - Prompt when overwriting or deleting local items
        - Upload changed files automatically to the default server -> Always
        - Delete remote files when local are deleted
        - Preserve original file permissions -> No
        - Warn when uploading over newer file -> No
8. Configure the default deployment server
    1. Open CLion's "search everywhere" window (can be opened via magnifying glass in upper right corner)
    2. Search "Show Default Deployment Server" -> set to "On"
    3. Click the now visible "Remote Development" on the bottom bar -> Remote Development -> AuraeVM
9. Upload the project to the VM
    1. Open the Project window -> right click the root of the project -> Deployment -> Upload to AuraeVM
        - *Changed files should automatically be uploaded, based on our settings, but CLion seems to use save events to
          trigger the upload. Manually triggering the upload like this should only be required when the project files
          change without CLion realizing, such as when switching git branches*
        - *Syncing only occurs from host to vm, but some files are generated on build. To sync from vm to host, open the
          Project window -> right click the root of the project -> Deployment -> Sync with Deployed to AuraeVM. (
          Generated files should not be checked in.)*

## Back to the VM (setting up Aurae)

*Some of these steps are documented at [Building from Source](https://aurae.io/build/). These steps are likely going to
get out of sync with the actual dependencies in the project. The GHA build
image [manifest file](https://github.com/aurae-runtime/aurae/blob/main/images/Dockerfile.build) is also a good source to
see the project's dependencies.*

1. Check the `Desktop/aurae` directory, it should no longer be empty
2. Install dependencies. In the terminal run...
    ```bash
    sudo apt-get update;
    sudo apt-get install -y protobuf-compiler;
    sudo apt-get install -y musl-tools;
    sudo apt-get install -y build-essential;
    sudo apt-get install -y llvm-15-dev; # bpf-linker dependency
    sudo apt-get install -y libclang-15-dev; # bpf-linker dependency
    ```
3. Install rust:
    1. Open the browser in the VM -> [https://rustup.rs](https://rustup.rs) -> copy the command (this is an official
       Rust project, so we trust the command)
    2. Run the command -> Select option 1 ("Proceed with installation (default)") when prompted
    3. Either follow the onscreen instructions to update PATH or close and reopen the terminal
    4. Run `rustup target add aarch64-unknown-linux-musl`
4. Install buf
    - [Buf Installation Docs](https://docs.buf.build/installation)
    - Homebrew is not an option
    - Binary works
        1. Open a terminal (the directory should not be in your `Desktop/aurae` directory; desktop is easiest)
        2. Run `touch buf.sh`
        3. Run `vi buf.sh`
        4. Copy + paste the code block on the buf website
        5. Hit ESC then ":wq" to save and exit
        6. Run `sudo -E sh buf.sh`
6. Build and install Aurae (terminal must be in the `Desktop/aurae` directory)
   ```bash
   make pki config;
   make build;
   ```
7. auraed needs to be run using sudo (`sudo -E auraed`), but Ubuntu will not let you for security reasons
    - Option A: use the full path: `sudo -E [full path to auraed installed by cargo]`
    - Option B: remove the security
        - Run `sudo -E visudo`
        - Comment out the line "Defaults secure_path=..." by prefixing it with "#" (disables security)
        - Save & exit via Ctrl+x -> Y -> ENTER
    - Option C: ease the security
        - add the path of the directory where cargo installed auraed to the same line indicated in option B.
