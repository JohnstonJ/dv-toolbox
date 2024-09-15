# dv-toolbox

dv-toolbox provides a set of tools for analyzing and repairing [DV](https://en.wikipedia.org/wiki/DV_(video_format)) format video files.

## Local development

### Windows

A quick start for developing and building Windows binaries using the provided [Vagrantfile](Vagrantfile) is below.  Using Vagrant will isolate the development toolchain to a virtual machine running an evaluation copy of Windows.  Both Visual Studio Code and Visual Studio Community are set up for development.

1. Install [Vagrant](https://developer.hashicorp.com/vagrant/downloads) from the downloads page.

2. Install [Hyper-V](https://learn.microsoft.com/en-us/virtualization/hyper-v-on-windows/quick-start/enable-hyper-v), [VMware](https://www.vmware.com/products/desktop-hypervisor/workstation-and-fusion), or [VirtualBox](https://www.virtualbox.org/wiki/Downloads).  Those are the Vagrant providers that are supported by the underlying [Vagrant box](https://github.com/gusztavvargadr/packer?tab=readme-ov-file#overview) used by this project.

    At this time, the rest of these instructions have only been tested with Hyper-V on a Windows 10 amd64 host.  If you are running Windows on the host PC, Hyper-V is a good choice.

3. Ensure that you have an appropriate [Synced Folder](https://developer.hashicorp.com/vagrant/docs/synced-folders) type set up.  SMB is recommended if the host supports it.

    - NOTE: Due to [issue 10661](https://github.com/hashicorp/vagrant/issues/10661), if your normal Windows username/password has spaces/punctuation (and you'd like to keep it that way), you will need to:
        1. Create a different local user on the Windows host PC that does not have spaces and use that when running `vagrant up`.
        2. Explicitly give this user access to the project/repository directory: go to the folder properties, go to the **Security** tab, and grant this user **Full control** over this repository **parent** directory.  For example, if this repository is located in `C:\Users\John\Documents\Projects\dv-toolbox`, then grant the user full access to the `Projects` directory.

4. Start the virtual machine.  For Hyper-V, this must be done from an administrator console:

    ```PowerShell
    vagrant up
    ```

    Vagrant will automatically choose a provider based on the [default provider search procedure](https://developer.hashicorp.com/vagrant/docs/providers/basic_usage#default-provider), but you can specify a different one using `--provider`.  If the SMB synced folder is used, you'll be asked to enter your username and password for the host PC.

5. You can connect to the VM using Remote Desktop.  Locate the IP address in Hyper-V Manager.  Alternatively, you can connect to the VM directly from Hyper-V Manager.  The username and password are both `vagrant`.

6. Alternatively, you can connect using SSH.
    1. Run `vagrant ssh-config` and add the SSH configuration to your `~/.ssh/config` file.  You may wish to rename the `Host` from `default` to something more specific.  We'll assume you call the host `dv-toolbox`.
    2. Test the SSH connection by running:

        ```PowerShell
        ssh dv-toolbox
        # <log in using username and password of: vagrant>
        ```

    3. Once logged in, test that `dir R:\dv-toolbox` shows the contents of the repository.
    4. If the virtual machine is restarted and gets a different IP address, you will have to repeat the SSH configuration step.

7. When you're finished, the virtual machine can be destroyed:

    ```PowerShell
    vagrant destroy
    ```

    You can also use `vagrant suspend` to suspend the VM, and `vagrant halt` to gracefully shut down the guest operating system.  These are useful if you're going to come back later to the virtual machine.  Finally, you can use `vagrant box list` and `vagrant box remove <box name>` to remove boxes that you no longer need.  You'll have to redownload them if you want to use them again later.

### Visual Studio Code

#### Starting Visual Studio Code

Once the virtual machine is running, enter an IDE as follows:

- Using Visual Studio Code in the virtual machine: double-click the icon on the desktop for that, and then install recommended extensions when prompted.

- Using Visual Studio Code remotely:

    1. Set up SSH as described above.

    2. On your host PC, install Visual Studio Code with the Remote - SSH extension if you haven't already.  Follow the [instructions](https://code.visualstudio.com/docs/remote/ssh#_connect-to-a-remote-host) to connect to the guest VM.

    3. Open the folder at `R:\dv-toolbox`.  Install recommended extensions when prompted.

### Visual Studio Community

In the virtual machine, double-click the icon on the desktop for that.  The extension will likely update rust-analyzer and want to restart Visual Studio at that time.
