!include "LogicLib.nsh"

!macro NSIS_HOOK_PREINSTALL

;---------------------------------------
; Kiểm tra agent có đang chạy không
;---------------------------------------

nsExec::ExecToStack 'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "try { Invoke-WebRequest http://127.0.0.1:15682/ping -UseBasicParsing | Out-Null; exit 0 } catch { exit 1 }"'
Pop $0
Pop $1

; Không kết nối được -> agent không chạy
${If} $0 != 0
    Goto Done
${EndIf}

;---------------------------------------
; Hỏi người dùng
;---------------------------------------

MessageBox MB_ICONQUESTION|MB_YESNO \
"vaOne Plugin đang chạy.$\r$\n$\r$\nĐóng ứng dụng để tiếp tục cài đặt?" \
IDYES CloseApp

Abort

CloseApp:

;---------------------------------------
; Yêu cầu agent tự thoát
;---------------------------------------

ExecWait 'taskkill /F /T /IM vaOne-agent.exe >nul 2>&1'

;---------------------------------------
; Chờ tối đa 15 giây
;---------------------------------------

StrCpy $2 30

WaitLoop:

    Sleep 500

    nsExec::ExecToStack 'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "try { Invoke-WebRequest http://127.0.0.1:15682/ping -UseBasicParsing | Out-Null; exit 0 } catch { exit 1 }"'
    Pop $3
    Pop $4

    ; ping thất bại => app đã tắt
    ${If} $3 != 0
        Goto Done
    ${EndIf}

    IntOp $2 $2 - 1

    ${If} $2 > 0
        Goto WaitLoop
    ${EndIf}

MessageBox MB_ICONSTOP \
"Không thể đóng vaOne Plugin.$\r$\nVui lòng đóng ứng dụng rồi chạy lại trình cài đặt."

Abort

Done:

!macroend