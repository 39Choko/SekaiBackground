$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$folderPath = "C:\39Choko\SekaiBackground"

function Test-Admin {
  [CmdletBinding()]
  param ()
  begin {
    Write-Host -Object "Checking for administrator privileges..."
  }
  process {
    $currentUser = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    -not $currentUser.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
  }
}

function Test-PowerShellVersion {
  [CmdletBinding()]
  param ()
  begin {
    $PSMinVersion = [Version]'5.1'
  }
  process {
    Write-Host -Object "Checking for PowerShell version $PSMinVersion or higher..."
    $PSVersionTable.PSVersion -ge $PSMinVersion
  }
}

function Get-SekaiBackground {
  [CmdletBinding()]
  param ()
  begin {
    Write-Host -Object "Downloading SekaiBackground from GitHub..."
    $lastestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/39Choko/SekaiBackground/releases/latest"
    $targetVersion = $lastestRelease.tag_name
    $binaryPath = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "sekai-bg.exe")
  }
  process {
    Write-Host -Object "Downloading SekaiBackground version $targetVersion..."
    $Parameters = @{
      Uri = "https://github.com/39Choko/SekaiBackground/releases/download/$targetVersion/sekai-bg.exe"
      UseBasicParsin = $true
      OutFile = $binaryPath
    }
    Invoke-WebRequest @Parameters
  }
  end {
    $binaryPath
  }
}

function Add-SekaiBackgroundToPath {
  [CmdletBinding()]
  param ()
  begin {
    Write-Host -Object "Adding SekaiBackground to system PATH..."
    $user = [EnvironmentVariableTarget]::User
    $path = [Environment]::GetEnvironmentVariable('PATH', $user)
  }
  process {
    if ($path -notlike "*$folderPath*") {
      $path = "$path;$folderPath"
    }
  }
  end {
    [Environment]::SetEnvironmentVariable('PATH', $path, $user)
    $env:PATH = $path
  }
}

function Install-SekaiBackground {
  [CmdletBinding()]
  param ()
  begin {
    Write-Host -Object "Installing SekaiBackground..."
  }
  process {
    $binaryPath = Get-SekaiBackground
    Move-Item -Path $binaryPath -Destination "$folderPath\sekai-bg.exe" -Force
    Add-SekaiBackgroundToPath
  }
}

if (-not (Test-PowerShellVersion)) {
  Write-Unsuccess
  Write-Warning -Message 'PowerShell 5.1 or higher is required to run this script'
  Write-Warning -Message "You are running PowerShell $($PSVersionTable.PSVersion)"
  Write-Host -Object 'PowerShell 5.1 install guide:'
  Write-Host -Object 'https://learn.microsoft.com/skypeforbusiness/set-up-your-computer-for-windows-powershell/download-and-install-windows-powershell-5-1'
  Write-Host -Object 'PowerShell 7 install guide:'
  Write-Host -Object 'https://learn.microsoft.com/powershell/scripting/install/installing-powershell-on-windows'
  Pause
  exit
}

if (-not (Test-Admin)) {
  Write-Warning -Message 'This script is running with administrator privileges. Please run it without admin rights.'
  $Host.UI.RawUI.FlushInputBuffer()
  $choices = [System.Management.Automation.Host.ChoiceDescription[]] @(
    (New-Object System.Management.Automation.Host.ChoiceDescription -ArgumentList '&Yes', 'Abort installation'),
    (New-Object System.Management.Automation.Host.ChoiceDescription -ArgumentList '&No', 'Resume installation')
  )
  $result = $Host.UI.PromptForChoice('', 'Do you want to abort the installation?', $choices, 0)
  if ($result -eq 0) {
    Write-Host -Object 'Installation aborted by user'
    Pause
    exit
  }
}

Install-SekaiBackground
Write-Host -Object 'SekaiBackground has been installed successfully!'