!macro NSIS_HOOK_POSTINSTALL
  CreateShortCut "$DESKTOP\漫畫書庫.lnk" "$INSTDIR\manga-shelf.exe"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$DESKTOP\漫畫書庫.lnk"
!macroend
