$files = Get-ChildItem "d:\ai-projects\mgit\mgit-core\tests\*.rs"
foreach ($file in $files) {
    $content = Get-Content $file.FullName -Encoding UTF8
    if ($content -match "mgit::utils::error") {
        $newContent = $content -replace "mgit::utils::error", "mgit::error"
        Set-Content -Path $file.FullName -Value $newContent -Encoding UTF8
        Write-Host "Updated $($file.Name)"
    }
}
