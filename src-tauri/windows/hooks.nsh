!include "LogicLib.nsh"

!macro NSIS_HOOK_PREINSTALL

;---------------------------------------
; Kiểm tra HTTP server có chạy không
;---------------------------------------

ClearErrors

nsExec::ExecToStack 'powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command "try { Invoke-WebRequest http://127.0.0.1:15682/ping -UseBasicParsing | Out-Null; exit 0 } catch { exit 1 }"'
Pop $0

; Nếu != 0 nghĩa là không kết nối được -> app không chạy
${If} $0 != 0
    Goto Done
${EndIf}

;---------------------------------------
; App đang chạy
;---------------------------------------

MessageBox MB_ICONQUESTION|MB_YESNO \
    "vaOne Plugin đang chạy.$\r$\n$\r$\nĐóng ứng dụng để tiếp tục cài đặt?" \
    IDYES CloseApp

Abort

CloseApp:

;---------------------------------------
; Yêu cầu app tự thoát
;---------------------------------------

nsExec::Exec 'powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command "try { Invoke-RestMethod -Uri ''http://127.0.0.1:15682/exit'' -Method POST | Out-Null } catch {}"'

;---------------------------------------
; Chờ tối đa 15 giây
;---------------------------------------

StrCpy $1 30

WaitLoop:

    Sleep 500

    nsExec::ExecToStack 'cmd /C tasklist /FI "IMAGENAME eq vaone-plugin.exe" | find /I "vaone-plugin.exe"'
    Pop $2
    Pop $3

    ; process đã biến mất
    ${If} $2 != 0
        Goto Done
    ${EndIf}

    IntOp $1 $1 - 1

    ${If} $1 > 0
        Goto WaitLoop
    ${EndIf}

;---------------------------------------
; Hết thời gian
;---------------------------------------

MessageBox MB_ICONSTOP \
    "Không thể đóng vaOne Plugin.$\r$\nVui lòng đóng ứng dụng rồi chạy lại trình cài đặt."

Abort

Done:

!macroend