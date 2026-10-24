$elf = 'target/armv4t-none-eabi/release/ipod'
$firmware = 'target/armv4t-none-eabi/release/linux.bin'

# no global ErrorActionPreference: it turns cargo's stderr progress into errors when output is redirected
Push-Location $PSScriptRoot
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw 'Build failed.' }

    arm-none-eabi-objcopy -O binary $elf $firmware
    if ($LASTEXITCODE -ne 0) { throw 'Binary conversion failed.' }

    $ipod = Get-PSDrive -PSProvider FileSystem |
        Where-Object { Test-Path -LiteralPath (Join-Path $_.Root 'iPod_Control') -PathType Container -ErrorAction SilentlyContinue } |
        Select-Object -First 1
    if (-not $ipod) { throw "iPod is not mounted; firmware remains at $firmware." }

    $destination = Join-Path $ipod.Root 'linux.bin'
    Copy-Item -LiteralPath $firmware -Destination $destination -Force -ErrorAction Stop
    Write-Host "Copied linux.bin to $destination"
}
finally {
    Pop-Location
}
