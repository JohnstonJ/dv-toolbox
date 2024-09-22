# -*- mode: ruby -*-
# vi: set ft=ruby :

# Manually reformatted with Prettier at https://normaltool.com/formatters/ruby-formatter

POWERSHELL_HEADER = <<~'EOT'
  $ErrorActionPreference = 'Stop'
  $ProgressPreference = 'SilentlyContinue'
  Set-StrictMode -Version 3.0
  function Check-Exit-Code {
    # 0x8A15002B is winget APPINSTALLER_CLI_ERROR_UPDATE_NOT_APPLICABLE (package already installed)
    if ($LASTEXITCODE -and $LASTEXITCODE -ne 0x8A15002B) { exit $LASTEXITCODE }
  }
EOT

PROJECT_NAME = "dv-toolbox"

Vagrant.configure("2") do |config|
  config.vm.box = "gusztavvargadr/windows-11-23h2-enterprise"

  # Share the parent directory as well: this works around Visual Studio's inability to open
  # a bare network share.  See:
  # https://developercommunity.visualstudio.com/t/Folder-View-is-empty-when-opening-a-fold/10745811
  # Also it is a good place to mount a network drive, while still having a subdirectory to work
  # with so we don't have to open the root drive path, which could also potentially have issues.
  # Using a network drive instead of the symlink created by Vagrant avoids yet more issues, like:
  # https://github.com/microsoft/vscode/issues/229661
  config.vm.synced_folder "..", "/repos"

  # Use another name for the default synced folder
  config.vm.synced_folder ".", "/#{PROJECT_NAME}"
  config.vm.synced_folder ".", "/vagrant", disabled: true

  config.vm.provider "hyperv" do |provider|
    provider.cpus = 4
    provider.memory = 1024
    provider.maxmemory = 12_288
    provider.linked_clone = true # saves a lot of time on first "vagrant up"
  end

  # Fix some of the most annoying default things in Windows
  config.vm.provision "shell_config",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    # Make the Windows 11 taskbar useful again

    # Left alignment
    Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' `
      -Name 'TaskbarAl' -Value 0 -Type DWord

    # Combine taskbar buttons and hide labels: When taskbar is full
    Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' `
      -Name 'TaskbarGlomLevel' -Value 1 -Type DWord
    # Combine taskbar buttons and hide labels on other taskbars: When taskbar is full
    Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' `
      -Name 'MMTaskbarGlomLevel' -Value 1 -Type DWord

    # When using multiple displays, show my taskbar apps on: Taskbar where window is open
    Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' `
      -Name 'MMTaskbarMode' -Value 2 -Type DWord

    # Widgets button: off
    New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Dsh' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\Software\Policies\Microsoft\Dsh' `
      -Name 'AllowNewsAndInterests' -Value 0 -Type DWord

    # Search: Hide
    Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Search' `
      -Name 'SearchboxTaskbarMode' -Value 0 -Type DWord

    # Other shell settings

    # Hide extensions for known file types: off
    Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' `
      -Name 'HideFileExt' -Value 0 -Type DWord

    # Show hidden files, folders, and drives
    Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' `
      -Name 'Hidden' -Value 1 -Type DWord

    # Underline access keys
    Set-ItemProperty -Path 'HKCU:\Control Panel\Accessibility\Keyboard Preference' `
      -Name 'On' -Value 1 -Type String
    # Set keyboard cues bit
    # https://web.archive.org/web/20090721134012/http://technet.microsoft.com/en-us/library/cc957204.aspx
    $upm = (
      Get-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'UserPreferencesMask'
    ).UserPreferencesMask
    $upm[0] = $upm[0] -bor 0x20
    Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' `
      -Name 'UserPreferencesMask' -Value $upm -Type Binary

    # Hide first run experience in Microsoft Edge
    New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' `
      -Name 'HideFirstRunExperience' -Value 1 -Type DWord
  EOT

  # Set up package managers
  config.vm.provision "package_managers",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    # The winget bundled with Windows 11 is old and broken, and not capable of
    # self-updating.  See any GitHub issue at https://github.com/microsoft/winget-cli/issues/
    # referencing error "0x8a15000f : Data required by the source is missing"
    # Instead, follow sandbox installation instructions to install from scratch:
    # https://learn.microsoft.com/en-us/windows/package-manager/winget/#install-winget-on-windows-sandbox
    # We do pin the winget version here, since the other dependencies are also pinned.
    Invoke-WebRequest -Uri https://github.com/microsoft/winget-cli/releases/download/v1.8.1911/Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle -OutFile Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle
    Invoke-WebRequest -Uri https://aka.ms/Microsoft.VCLibs.x64.14.00.Desktop.appx -OutFile Microsoft.VCLibs.x64.14.00.Desktop.appx
    Invoke-WebRequest -Uri https://github.com/microsoft/microsoft-ui-xaml/releases/download/v2.8.6/Microsoft.UI.Xaml.2.8.x64.appx -OutFile Microsoft.UI.Xaml.2.8.x64.appx
    # 0x80073D06 is " The package could not be installed because a higher
    # version of this package is already installed."
    try {
      Add-AppxPackage Microsoft.VCLibs.x64.14.00.Desktop.appx
    } catch { if ($_.exception.Message -notmatch "0x80073D06") { throw } }
    try {
      Add-AppxPackage Microsoft.UI.Xaml.2.8.x64.appx
    } catch { if ($_.exception.Message -notmatch "0x80073D06") { throw } }
    try {
      Add-AppxPackage Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle
    } catch { if ($_.exception.Message -notmatch "0x80073D06") { throw } }
  EOT

  # Install a newer PowerShell + Windows Terminal
  config.vm.provision "shell_terminal",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    winget install --id Microsoft.PowerShell --silent `
      --accept-source-agreements --accept-package-agreements
    Check-Exit-Code

    winget install --id Microsoft.WindowsTerminal --silent `
      --accept-source-agreements --accept-package-agreements
    Check-Exit-Code
    # Make Windows Terminal the default
    New-Item -Path 'HKCU:\Console\%%Startup' -Force | Out-Null
    Set-ItemProperty -Path 'HKCU:\Console\%%Startup' `
      -Name 'DelegationConsole' -Value '{2EACA947-7F5F-4CFA-BA87-8F7FBEEFBE69}' -Type String
    Set-ItemProperty -Path 'HKCU:\Console\%%Startup' `
      -Name 'DelegationTerminal' -Value '{E12CFF52-A866-4C77-9A90-F570A7AA2C6B}' -Type String
  EOT

  # Install Git for Windows and copy user's settings
  config.vm.provision "git", type: "shell", inline: POWERSHELL_HEADER + <<~'EOT'
    # https://github.com/microsoft/winget-pkgs/issues/173310
    winget install --id Git.Git --silent `
      --accept-source-agreements --accept-package-agreements `
      --custom "/COMPONENTS=gitlfs,windowsterminal"
    Check-Exit-Code
  EOT

  config.vm.provision "git_config",
                      type: "file",
                      source: "~/.gitconfig",
                      destination: "C:/Users/vagrant/.gitconfig"

  config.vm.provision "git_ownership",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    # Work around "dubious ownership" error when working over SMB shares
    git config --global --add safe.directory '*'
    Check-Exit-Code
  EOT

  # Install Visual Studio: required by Rust
  config.vm.provision "visual_studio",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    # Install the full IDE for use with rust-analyzer.vs:
    winget install --id Microsoft.VisualStudio.2022.Community --silent `
      --accept-source-agreements --accept-package-agreements `
      --override "--quiet --wait --add Microsoft.VisualStudio.Workload.NativeDesktop;includeRecommended"

    # Alternatively, just the build tools:
    #winget install --id Microsoft.VisualStudio.2022.BuildTools --silent `
    #  --accept-source-agreements --accept-package-agreements `
    #  --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended"

    Check-Exit-Code
  EOT

  # Install Rustup / Rust
  config.vm.provision "rustup_bootstrap",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    winget install --id Rustlang.Rustup --silent `
      --accept-source-agreements --accept-package-agreements
    Check-Exit-Code
  EOT

  # In case the winget package is a little out-of-date, get the latest rust:
  config.vm.provision "rustup_update",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    rustup update
    Check-Exit-Code
  EOT

  # Install just: a command runner
  config.vm.provision "just",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    winget install --id Casey.Just --exact --silent `
      --accept-source-agreements --accept-package-agreements
    Check-Exit-Code
  EOT

  # Install vcpkg and make it globally available
  # (Note it's also included with Visual Studio Community, but not the simple VS Build Tools)
  config.vm.provision "vcpkg",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    $vcpkg_root = Join-Path $HOME ".vcpkg"
    if (-not (Test-Path $vcpkg_root)) {
      git clone https://github.com/microsoft/vcpkg.git $vcpkg_root
      Check-Exit-Code
    }
    Set-Location $vcpkg_root
    git pull
    Check-Exit-Code

    .\bootstrap-vcpkg.bat
    Check-Exit-Code

    .\vcpkg integrate install
    Check-Exit-Code

    [Environment]::SetEnvironmentVariable("VCPKG_ROOT", $vcpkg_root, "User")
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -eq $null) {
      [Environment]::SetEnvironmentVariable("Path", $vcpkg_root, "User")
    } elseif ($UserPath -split ";" -notcontains $vcpkg_root) {
      [Environment]::SetEnvironmentVariable("Path", "$vcpkg_root;$UserPath", "User")
    }
  EOT

  # Install LLVM via winget, as recommended by the rust-bindgen documentation.  Note that we could
  # also cleanly compile from source using vcpkg and keep it local to the project directory.  But
  # that takes over 2 hours to compile on my laptop (using x64-windows-release triplet and
  # clang/default-targets features).  Not worth it.
  config.vm.provision "llvm",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    winget install --id LLVM.LLVM --exact --silent `
      --accept-source-agreements --accept-package-agreements
    Check-Exit-Code

    $install_dir = (Get-ItemProperty -Path 'HKLM:\SOFTWARE\WOW6432Node\LLVM\LLVM')."(default)"
    # rust bindgen expects this environment variable to be sett
    $install_dir = Join-Path $install_dir "bin"
    [Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $install_dir, "Machine")
  EOT

  # Install Visual Studio Code
  config.vm.provision "vscode",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    winget install --id Microsoft.VisualStudioCode --scope machine --silent `
      --accept-source-agreements --accept-package-agreements
    Check-Exit-Code
  EOT

  # Install the Rust development pack for Visual Studio
  # (In theory, our .vsconfig file should prompt the user to install it, but that mechanism
  # isn't quite working yet with this extension.)
  config.vm.provision "rust_analyzer_vs",
                      type: "shell",
                      inline: POWERSHELL_HEADER + <<~'EOT'
    # from https://gist.github.com/ScottHutchinson/b22339c3d3688da5c9b477281e258400
    $Uri = "https://marketplace.visualstudio.com/items?itemName=kitamstudios.RustDevelopmentPack"
    $HTML = Invoke-WebRequest -Uri $Uri -UseBasicParsing -SessionVariable session
    $FileUri = $HTML.Links | `
      Where-Object { $_.PSObject.Properties.Match('class').Count -and `
        $_.class -eq "install-button-container" } | `
      Select-Object -ExpandProperty href
    if (-not $FileUri) {
      Write-Error "Could not find download link on the VS extensions page."
      exit 1
    }
    $FileUri = "https://marketplace.visualstudio.com$($FileUri)"
    Write-Host "Downloading extension from $($FileUri)..."
    Invoke-WebRequest $FileUri -OutFile "C:\rustdev.vsix" -WebSession $session
    Write-Host "Installing extension..."
    $VSIXInstaller = "C:\Program Files (x86)\Microsoft Visual Studio\Installer\resources\app\" + `
      "ServiceHub\Services\Microsoft.VisualStudio.Setup.Service\VSIXInstaller.exe"
    $p = Start-Process $VSIXInstaller -ArgumentList "/q /a C:\rustdev.vsix" -Wait -PassThru
    if ($p.ExitCode) { exit $p.ExitCode }
    Remove-Item "C:\rustdev.vsix"
  EOT

  # Required if you want to access the Vagrant SMB shares from SSH (e.g. if using VS Code
  # remotely), since we can't access the credentials set up by Vagrant via CMDKEY.  Password
  # authentication as an alternative does not work:
  # https://github.com/PowerShell/Win32-OpenSSH/issues/2273
  class SMBUsername
    # approach from https://github.com/hashicorp/vagrant/issues/2662#issuecomment-328838768
    def to_s
      print "This will create a Windows PowerShell profile that runs NET USE to mount\n"
      print "the Vagrant SMB shares when connecting via SSH.  Enter the same credentials\n"
      print "that you entered earlier for connecting the SMB synced folders when Vagrant\n"
      print "asked you for them.\n"
      print "SMB folder username ([domain\]user): "
      STDIN.gets.chomp
    end
  end

  class SMBPassword
    def to_s
      print "SMB folder password: "
      STDIN.gets.chomp
    end
  end

  # Always run this in case the SMB hostname changes during "vagrant up".
  config.vm.provision "ssh_smb_password",
                      type: "shell",
                      run: "always",
                      env: {
                        "SMB_USERNAME" => SMBUsername.new,
                        "SMB_PASSWORD" => SMBPassword.new,
                        "SMB_LINK" => "C:\\repos"
                      },
                      inline: POWERSHELL_HEADER + <<~'EOT'
    # Get SMB target
    $Target = Get-Item $env:SMB_LINK | Select-Object -ExpandProperty Target
    $Target = $Target -replace "^UNC", "\"  # change UNC\host\share to \\host\share
    if (-not ($Target -match '^\\\\(?<host>.+)\\')) {
      Write-Host "Could not identify UNC hostname from SMB folder sync symlink."
      Write-Host "We will assume that you are not using SMB synced folders with Vagrant."
      exit 0
    }
    $SmbHost = $Matches.host

    # Create Windows PowerShell profile
    New-Item -Path 'C:\Users\vagrant\Documents\WindowsPowerShell' -ItemType 'directory' -Force `
      | Out-Null
    Set-Content -Path 'C:\Users\vagrant\Documents\WindowsPowerShell\Profile.ps1' `
      -Value ( `
        "(net view `"\\$($SmbHost)`" 2>&1) | Out-Null`n" + `
        "if (`$LASTEXITCODE) {`n" + `
        "  net use `"\\$($SmbHost)`" `"$($env:SMB_PASSWORD)`" /USER:`"$($env:SMB_USERNAME)`" " + `
        "| Out-Null`n" + `
        "}`n" + `
        "# Persisted network drives are not restored in SSH connections, so reconnect here:`n" + `
        "if (-not (Test-Path `"R:\`")) {`n" + `
        "  New-SmbMapping R: `"$($Target)`" | Out-Null`n" + `
        "}" `
      )

    # Persist the mapped network drive.  Persistence doesn't work in SSH, but it's still useful
    # in interactive scenarios (e.g. Remote Desktop / interactive login):
    # https://github.com/PowerShell/Win32-OpenSSH/issues/1734
    if (Test-Path 'R:\') {
      Remove-SmbMapping R: -Force | Out-Null
    }
    New-SmbMapping R: $Target -Persistent $true | Out-Null
  EOT

  # Create shortcuts on the desktop
  config.vm.provision "shortcuts",
                      type: "shell",
                      env: {
                        "PROJECT_NAME" => PROJECT_NAME
                      },
                      inline: POWERSHELL_HEADER + <<~'EOT'
    function New-Shortcut {
      param ($Name, $TargetPath, $Arguments)
      $ShortcutPath = "C:\Users\vagrant\Desktop\$($Name).lnk"
      $s = (New-Object -ComObject WScript.Shell).CreateShortcut($ShortcutPath)
      $s.TargetPath = $TargetPath
      $s.Arguments = $Arguments
      $s.Save()
    }
    New-Shortcut -Name "$($env:PROJECT_NAME) VSCode" `
      -TargetPath "C:\Program Files\Microsoft VS Code\Code.exe" `
      -Arguments "R:\$($env:PROJECT_NAME)"
    New-Shortcut -Name "$($env:PROJECT_NAME) Visual Studio" `
      -TargetPath ("C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\IDE\" + `
        "devenv.exe") `
      -Arguments "R:\$($env:PROJECT_NAME)"
    New-Shortcut -Name "$($env:PROJECT_NAME) Folder" `
      -TargetPath "R:\$($env:PROJECT_NAME)" -Arguments $null
  EOT

  # Reboot at the end to ensure all changes take effect.
  config.vm.provision "reboot",
                      type: "shell",
                      reboot: true,
                      inline: POWERSHELL_HEADER + <<~'EOT'
    Write-Host "Rebooting..."
  EOT
end
