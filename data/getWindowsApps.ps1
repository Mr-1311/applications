# script taken from https://github.com/oliverschwendener/ueli
param(
    [string[]]$FolderPaths,
    [string[]]$FileExtensions
)

function Get-WindowsApps {
    param(
        [string[]]$FolderPaths,
        [string[]]$FileExtensions
    )

    Get-ChildItem -File -Path $FolderPaths -Recurse -Include $FileExtensions | Select-Object -Property Name, FullName, Extension, BaseName | ConvertTo-Json
}

# Call the function with script parameters
Get-WindowsApps -FolderPaths $FolderPaths -FileExtensions $FileExtensions