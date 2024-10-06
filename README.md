# dv-toolbox

dv-toolbox provides a set of tools for analyzing and repairing [DV](https://en.wikipedia.org/wiki/DV_(video_format)) format video files.

## Local development

Unless you specifically need to test changes against a particular platform, it's recommended to develop for Linux.  Debug build compiling and linking is the fastest there.

### Linux

A quick start for developing and building Linux binaries using the provided [Development Container](.devcontainer/devcontainer.json) with Visual Studio Code is below.  This will isolate the development toolchain to a Docker container.

Supported hosts:

- CPU architecture:
  - `amd64`
- Operating systems:
  - Windows
  - Linux

At this time, macOS on `arm64` isn't supported yet as a host, but it would be relatively easy to adapt the project to support it.

1. Windows hosts: install [WSL 2](https://learn.microsoft.com/en-us/windows/wsl/install).  (Docker Desktop can also use Hyper-V, but it's basically [deprecated](https://www.docker.com/blog/docker-hearts-wsl-2/).)

2. Install Docker [Desktop](https://docs.docker.com/desktop/) (Mac/Windows/Linux) or [Engine](https://docs.docker.com/engine/install/) (Linux only) by following the installation instructions.

3. Follow the instructions for [Visual Studio Code](#visual-studio-code) to open the project there.  This will build and run the docker image/container used for development.

Connection options when using the Dev Container:

- Open a terminal in Visual Studio Code as normal
- Run a shell in the running container:

  ```bash
  # List docker containers
  docker ps
  # Open a shell in the container you identified as the one started by VS Code
  docker exec -it <container ID or name> bash
  ```

- Connect to a graphical environment by navigating to [http://localhost:6080/](http://localhost:6080/).

### Windows

A quick start for developing and building Windows binaries using the provided [Vagrantfile](Vagrantfile) is below.  Using Vagrant will isolate the development toolchain to a virtual machine running an evaluation copy of Windows.  Both Visual Studio Code and Visual Studio Community are set up for development.

Supported hosts:

- CPU architectures:
  - `amd64` only
  - Not practical to support at this time: `arm64` processors due to lack of Vagrant support.
- Operating systems:
  - Windows
  - Linux (any distribution supported by Vagrant)hosts with `amd64` CPU architecture.

Instructions:

1. Install [Vagrant](https://developer.hashicorp.com/vagrant/downloads) from the downloads page.

2. Install [Hyper-V](https://learn.microsoft.com/en-us/virtualization/hyper-v-on-windows/quick-start/enable-hyper-v), [VMware](https://www.vmware.com/products/desktop-hypervisor/workstation-and-fusion), or [VirtualBox](https://www.virtualbox.org/wiki/Downloads).  Those are the Vagrant providers that are supported by the underlying [Vagrant box](https://github.com/gusztavvargadr/packer?tab=readme-ov-file#overview) used by this project.

    At this time, the rest of these instructions have only been tested with Hyper-V on a Windows 10 amd64 host.  If you are running Windows on the host PC, Hyper-V is a good choice.

3. Ensure that you have an appropriate [Synced Folder](https://developer.hashicorp.com/vagrant/docs/synced-folders) type set up.  SMB is recommended if the host supports it.

    - NOTE: Due to [issue 10661](https://github.com/hashicorp/vagrant/issues/10661), if your normal Windows username/password has spaces/punctuation (and you'd like to keep it that way), you will need to:
        1. Create a different local user on the Windows host PC that does not have spaces and use that when running `vagrant up`.
        2. Explicitly give this user access to the project/repository directory: go to the folder properties, go to the **Security** tab, and grant this user **Full control** over this repository **parent** directory.  For example, if this repository is located in `C:\Users\John\Documents\Projects\dv-toolbox`, then grant the user full access to the `Projects` directory.

4. Consider temporarily hacking the [`Vagrantfile`](Vagrantfile) to have a higher CPU count appropriate for your machine.

5. Start the virtual machine.  For Hyper-V, this must be done from an administrator console:

    ```PowerShell
    vagrant up
    ```

    Vagrant will automatically choose a provider based on the [default provider search procedure](https://developer.hashicorp.com/vagrant/docs/providers/basic_usage#default-provider), but you can specify a different one using `--provider`.  If the SMB synced folder is used, you'll be asked to enter your username and password for the host PC.

6. Connection options:
    - [Visual Studio Code](#visual-studio-code): most of the time, you'll connect remotely to the container from the host using Visual Studio Code, as documented in the linked section.
    - Remote Desktop:  Locate the IP address in Hyper-V Manager and connect using Remote Desktop.
    - Directly work on the VM's display output: connect to the VM directly from Hyper-V Manager.
    - SSH: you can connect using SSH.  Using Windows PowerShell is required for the mounted repository to work.
      1. Run `vagrant ssh-config` and add the SSH configuration to your `~/.ssh/config` file.  You may wish to rename the `Host` from `default` to something more specific.  We'll assume you call the host `dv-toolbox`.
      2. Test the SSH connection by running:

          ```PowerShell
          ssh dv-toolbox powershell
          # <log in using username and password of: vagrant>
          ```

      3. Once logged in, test that `dir R:\dv-toolbox` shows the contents of the repository.
      4. If the virtual machine is restarted and gets a different IP address, you will have to repeat the SSH configuration step.

    When connecting, the username and password are both `vagrant`.

7. When you're finished, the virtual machine can be destroyed:

    ```PowerShell
    vagrant destroy
    ```

    You can also use `vagrant suspend` to suspend the VM, and `vagrant halt` to gracefully shut down the guest operating system.  These are useful if you're going to come back later to the virtual machine.  Finally, you can use `vagrant box list` and `vagrant box remove <box name>` to remove boxes that you no longer need.  You'll have to redownload them if you want to use them again later.

Performance notes:

- Use [Winaero Tweaker] inside the virtual machine to disable Microsoft Defender / Windows Security.
- On the host PC, be sure to add the project directory as an exclusion to Windows Security.

### Checking and building code

This project uses [just](https://just.systems/man/en/) as a task runner.  To begin, open a shell to the project directory in the development environment.  For Windows builds, connect to the virtual machine via SSH and run `cd R:\dv-toolbox`.

Most common tasks:

- `just`: List all available recipes.
- `just base-deps`: Install C++ packages and binary tools.  Must be manually run before other recipes below, with the exception of `just check`.
- `just verify`: Run all checks and build all packages with the debug profile.
- `just coverage`: Test all packages with code coverage.  An HTML report is generated, along with an `lcov` file that the Visual Studio Code Coverage Gutters extension is configured to look for.
- `just test`: Test all packages; you may also pass a test name as a recipe parameter.
- `just doc`: Build crate documentation.
- `just fmt`: Format all code (will modify files).
- `just build-release`: Build default packages with the release profile.

### Visual Studio Code

1. Install [Visual Studio Code](https://code.visualstudio.com/download) on your host.  (You can also use it [portably](https://code.visualstudio.com/docs/editor/portable).)

   - If developing for Windows, you may skip this step if you want to run the Visual Studio Code inside the virtual machine using Remote Desktop.

2. Install the appropriate extension on the host:

   - Windows development via Vagrant: install the [Remote - SSH](https://code.visualstudio.com/docs/remote/ssh#_installation) extension via the linked instructions.
     - You can skip this if you are running Visual Studio Code inside the VM using Remote Desktop.
   - Linux development via Development Containers: install the [Dev Containers](https://code.visualstudio.com/docs/devcontainers/containers#_installation) extension via the linked instructions.

3. Windows development when using Vagrant: set up SSH access as described in the section above.

4. Open the project in Visual Studio Code:

   - Windows development via Vagrant - pick one of these alternatives:
     - Follow the [instructions](https://code.visualstudio.com/docs/remote/ssh#_connect-to-a-remote-host) to connect to the guest VM via SSH from the Visual Studio Code instance running on the host.  Open the folder at `R:\dv-toolbox`.
     - Alternatively, run the full VS Code GUI inside the VM using Remote Desktop.  Connect to the VM using Remote Desktop and double-click the shortcut icon on the desktop to open the project.
   - Linux development via Dev Containers: run the **Dev Containers: Open Folder in Container** command, and open the project.

5. Open the terminal in Visual Studio Code and run the following command:

   ```bash
   just vscode-setup
   ```

6. Install the extensions that are recommended by the workplace.

7. Install these additional extensions, depending on your platform.  This will enable Rust/C++ debugging, as well as C++ editing (e.g. for viewing FFmpeg source code):
   - Windows development: [C/C++ for Visual Studio Code](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools), from Microsoft
   - Linux development
     - [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb), from Vadim Chugunov
     - [clangd](https://marketplace.visualstudio.com/items?itemName=llvm-vs-code-extensions.vscode-clangd), from LLVM

8. Make a trivial change to `.vscode/tabaqa.json` and save it.  This will trigger tabaqa to regenerate the `.vscode/settings.json` file with settings for the newly-installed extensions.  Verify that a new `.vscode/settings.json` file now includes the settings in the generated `.vscode/rust-environment.json` file.  The latter was generated by `just vscode-setup`.

   Explanation: This project uses the [tabaqa](https://marketplace.visualstudio.com/items?itemName=KalimahApps.tabaqa) extension to merge user settings, repo-committed settings, and auto-generated settings into a final `.vscode/settings.json` file.  The rust-analyzer extension will not work correctly without this mechanism.

### Visual Studio Community

In the virtual machine, double-click the icon on the desktop for that.  The extension will likely update rust-analyzer and want to restart Visual Studio at that time.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
