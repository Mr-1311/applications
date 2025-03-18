# script taken from https://github.com/oliverschwendener/ueli
param(
    [string]$InFilePath,
    [string]$OutFilePath
)

function Get-Shortcut-Target {
    param([string]$ShortcutFilePath)

    try {
        $Shell = New-Object -ComObject WScript.Shell
        $TargetPath = $Shell.CreateShortcut($ShortcutFilePath).TargetPath
        $TargetPathAccessible = Test-Path -Path $TargetPath -PathType Leaf
        if ($TargetPathAccessible) {
            return $TargetPath
        }
        else {
            return $ShortcutFilePath
        }
    }
    catch {
        return $ShortcutFilePath
    }
}

function Get-Associated-Icon {
    param(
        [string]$InFilePath,
        [string]$OutFilePath
    )

    $ErrorActionPreference = "SilentlyContinue"
    Add-Type -AssemblyName System.Drawing

    if ($InFilePath.EndsWith(".lnk")) {
        $InFilePath = Get-Shortcut-Target -ShortcutFilePath $InFilePath
    }

    $Icon = [System.Drawing.Icon]::ExtractAssociatedIcon($InFilePath)

    if ($null -ne $Icon) {
        $Icon.ToBitmap().Save($OutFilePath, [System.Drawing.Imaging.ImageFormat]::Png)
    }
}

# Call the function with script parameters
Get-Associated-Icon -InFilePath $InFilePath -OutFilePath $OutFilePath