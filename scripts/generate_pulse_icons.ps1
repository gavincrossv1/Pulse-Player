# scripts/generate_pulse_icons.ps1
# 使用 .NET System.Drawing 原生生成 Pulse Player 全套高保真多分辨率应用图标与 Windows .ico

Add-Type -AssemblyName System.Drawing

function Draw-PulseIcon([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    
    # 最高抗锯齿与渲染品质
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)

    $scale = [float]$size / 24.0

    # 1. 绘制外部圆角视窗底座
    $pad = [float](1.8 * $scale)
    $rectW = [float]($size - 2.0 * $pad)
    $rectH = [float]($size - 2.0 * $pad)
    $radius = [float](5.5 * $scale)

    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $diameter = [float]($radius * 2.0)
    $path.AddArc($pad, $pad, $diameter, $diameter, 180, 90)
    $path.AddArc($pad + $rectW - $diameter, $pad, $diameter, $diameter, 270, 90)
    $path.AddArc($pad + $rectW - $diameter, $pad + $rectH - $diameter, $diameter, $diameter, 0, 90)
    $path.AddArc($pad, $pad + $rectH - $diameter, $diameter, $diameter, 90, 90)
    $path.CloseFigure()

    # 底座暗部微渐变填充
    $pt0 = New-Object System.Drawing.PointF(0.0, 0.0)
    $ptEnd = New-Object System.Drawing.PointF([float]$size, [float]$size)
    $cBg1 = [System.Drawing.Color]::FromArgb(245, 34, 25, 36)
    $cBg2 = [System.Drawing.Color]::FromArgb(250, 18, 15, 22)
    $bgBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush($pt0, $ptEnd, $cBg1, $cBg2)
    $g.FillPath($bgBrush, $path)
    $bgBrush.Dispose()

    # 外框霓虹粉渐变描边
    $cSt1 = [System.Drawing.Color]::FromArgb(255, 255, 46, 136)
    $cSt2 = [System.Drawing.Color]::FromArgb(255, 251, 114, 153)
    $strokeBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush($pt0, $ptEnd, $cSt1, $cSt2)
    $pen = New-Object System.Drawing.Pen($strokeBrush, [float][Math]::Max(1.0, 1.8 * $scale))
    $g.DrawPath($pen, $path)
    $pen.Dispose()
    $strokeBrush.Dispose()
    $path.Dispose()

    # 2. 绘制内部脉冲播放核 (Pulse Blade)
    # 上光刃
    $u1 = New-Object System.Drawing.PointF([float](8.2 * $scale), [float](6.8 * $scale))
    $u2 = New-Object System.Drawing.PointF([float](16.8 * $scale), [float](11.2 * $scale))
    $u3 = New-Object System.Drawing.PointF([float](14.6 * $scale), [float](13.9 * $scale))
    $u4 = New-Object System.Drawing.PointF([float](9.8 * $scale), [float](9.9 * $scale))
    $upperPoints = [System.Drawing.PointF[]]@($u1, $u2, $u3, $u4)

    $uPt0 = New-Object System.Drawing.PointF([float](8.0 * $scale), [float](6.0 * $scale))
    $uPt1 = New-Object System.Drawing.PointF([float](17.0 * $scale), [float](13.0 * $scale))
    $cUp1 = [System.Drawing.Color]::FromArgb(255, 255, 60, 140)
    $cUp2 = [System.Drawing.Color]::FromArgb(255, 255, 140, 180)
    $upperBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush($uPt0, $uPt1, $cUp1, $cUp2)
    $g.FillPolygon($upperBrush, $upperPoints)
    $upperBrush.Dispose()

    # 下光刃
    $l1 = New-Object System.Drawing.PointF([float](8.2 * $scale), [float](12.2 * $scale))
    $l2 = New-Object System.Drawing.PointF([float](13.6 * $scale), [float](15.8 * $scale))
    $l3 = New-Object System.Drawing.PointF([float](10.8 * $scale), [float](17.6 * $scale))
    $l4 = New-Object System.Drawing.PointF([float](9.4 * $scale), [float](16.8 * $scale))
    $l5 = New-Object System.Drawing.PointF([float](9.4 * $scale), [float](13.8 * $scale))
    $lowerPoints = [System.Drawing.PointF[]]@($l1, $l2, $l3, $l4, $l5)

    $lPt0 = New-Object System.Drawing.PointF([float](8.0 * $scale), [float](12.0 * $scale))
    $lPt1 = New-Object System.Drawing.PointF([float](14.0 * $scale), [float](18.0 * $scale))
    $cLw1 = [System.Drawing.Color]::FromArgb(240, 240, 35, 120)
    $cLw2 = [System.Drawing.Color]::FromArgb(240, 251, 114, 153)
    $lowerBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush($lPt0, $lPt1, $cLw1, $cLw2)
    $g.FillPolygon($lowerBrush, $lowerPoints)
    $lowerBrush.Dispose()

    $g.Dispose()
    return $bmp
}

# 导出 Windows 标准多图层 .ico
function Export-MultiSizeIco([string]$outputPath, [int[]]$sizes) {
    $bitmaps = @()
    foreach ($sz in $sizes) {
        $bitmaps += Draw-PulseIcon -size $sz
    }

    $ms = New-Object System.IO.MemoryStream
    $bw = New-Object System.IO.BinaryWriter($ms)

    # ICONDIR
    $bw.Write([uint16]0) # Reserved
    $bw.Write([uint16]1) # Type: 1 for ICO
    $bw.Write([uint16]$sizes.Count) # Image count

    $imageDataList = @()
    foreach ($bmp in $bitmaps) {
        $imgStream = New-Object System.IO.MemoryStream
        $bmp.Save($imgStream, [System.Drawing.Imaging.ImageFormat]::Png)
        $imageDataList += ,$imgStream.ToArray()
        $imgStream.Dispose()
    }

    $offset = 6 + ($sizes.Count * 16)
    for ($i = 0; $i -lt $sizes.Count; $i++) {
        $sz = $sizes[$i]
        $w = if ($sz -ge 256) { [byte]0 } else { [byte]$sz }
        $h = if ($sz -ge 256) { [byte]0 } else { [byte]$sz }
        $data = $imageDataList[$i]

        $bw.Write($w) # Width
        $bw.Write($h) # Height
        $bw.Write([byte]0) # ColorCount
        $bw.Write([byte]0) # Reserved
        $bw.Write([uint16]1) # Color planes
        $bw.Write([uint16]32) # Bits per pixel
        $bw.Write([uint32]$data.Length) # Image size in bytes
        $bw.Write([uint32]$offset) # Offset
        $offset += $data.Length
    }

    for ($i = 0; $i -lt $sizes.Count; $i++) {
        $data = $imageDataList[$i]
        $bw.Write($data)
    }

    $bw.Flush()
    [System.IO.File]::WriteAllBytes($outputPath, $ms.ToArray())
    $bw.Dispose()
    $ms.Dispose()

    foreach ($bmp in $bitmaps) {
        $bmp.Dispose()
    }
}

$iconsDir = "src-tauri\icons"
Write-Host ">>> 正在生成 Pulse Player 全套尺寸图标..." -ForegroundColor Cyan

# 1. 生成所有标准 png
$pngSizes = @{
    "32x32.png" = 32
    "128x128.png" = 128
    "128x128@2x.png" = 256
    "icon.png" = 512
    "Square30x30Logo.png" = 30
    "Square44x44Logo.png" = 44
    "Square71x71Logo.png" = 71
    "Square89x89Logo.png" = 89
    "Square107x107Logo.png" = 107
    "Square142x142Logo.png" = 142
    "Square150x150Logo.png" = 150
    "Square284x284Logo.png" = 284
    "Square310x310Logo.png" = 310
    "StoreLogo.png" = 50
}

foreach ($entry in $pngSizes.GetEnumerator()) {
    $targetPath = Join-Path $iconsDir $entry.Key
    $bmp = Draw-PulseIcon -size $entry.Value
    $bmp.Save($targetPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host "  -> 已生成 $targetPath ($($entry.Value)x$($entry.Value))"
}

# 2. 生成包含多级分辨率的 icon.ico
$icoPath = Join-Path $iconsDir "icon.ico"
Export-MultiSizeIco -outputPath $icoPath -sizes @(256, 128, 64, 48, 32, 24, 16)
Write-Host "  -> 已生成高质量全分辨率 Windows $icoPath" -ForegroundColor Green

# 3. 复制到 public 与 release 目录
$publicIco = "public\favicon.ico"
[System.IO.File]::Copy($icoPath, $publicIco, $true)
Write-Host "  -> 已同步 $publicIco" -ForegroundColor Green

Write-Host ">>> Pulse Player 图标全部生成完成！" -ForegroundColor Green
