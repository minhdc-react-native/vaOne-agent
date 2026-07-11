@echo off
setlocal

REM ==============================
REM Config
REM ==============================

REM Thư mục hiện tại của file bat
set "SCRIPT_DIR=%~dp0"

REM Thư mục bundle/nsis
set "BUNDLE=%SCRIPT_DIR%..\target\release\bundle\nsis"

set "GITHUB_USER=minhdc-react-native"
set "REPO=vaOne-update"
set "NOTES=- Sửa lỗi đồng bộ dữ liệu\n- Cải thiện hiệu năng\n- Khắc phục lỗi in hóa đơn"

powershell -NoProfile -ExecutionPolicy Bypass ^
"$bundle='%BUNDLE%';" ^
"$user='%GITHUB_USER%';" ^
"$repo='%REPO%';" ^
"$notes='%NOTES%';" ^
"" ^
"$exe=Get-ChildItem $bundle -Filter '*-setup.exe' | Select-Object -First 1;" ^
"if(!$exe){throw 'Setup exe not found'};" ^
"" ^
"$version=[regex]::Match($exe.Name,'_([0-9]+\.[0-9]+\.[0-9]+)_x64-setup\.exe').Groups[1].Value;" ^
"if(!$version){throw 'Cannot parse version'};" ^
"" ^
"$sig=Get-Content ($exe.FullName+'.sig') -Raw;" ^
"$sig=$sig.Trim();" ^
"" ^
"$url='https://github.com/'+$user+'/'+$repo+'/releases/download/v'+$version+'/'+$exe.Name;" ^
"" ^
"$obj=@{" ^
"version=$version;" ^
"notes=$notes;" ^
"pub_date=(Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ');" ^
"platforms=@{" ^
"'windows-x86_64'=@{" ^
"signature=$sig;" ^
"url=$url" ^
"}" ^
"}" ^
"};" ^
"" ^
"$obj | ConvertTo-Json -Depth 5 | Out-File ($bundle+'\latest.json') -Encoding utf8;" ^
"Write-Host '';" ^
"Write-Host '==============================';" ^
"Write-Host 'latest.json created';" ^
"Write-Host 'Version :' $version;" ^
"Write-Host 'Output  :' ($bundle+'\latest.json');" ^
"Write-Host '==============================';"

pause