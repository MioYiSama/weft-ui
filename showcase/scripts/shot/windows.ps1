# Run on the Parallels guest through /tmp/weft-win.sh, after /tmp/win-sync.ps1.
$checkout = 'C:\weft\repo'
$hostCheckout = '\\Mac\Home\Documents\Code\Projects\weft-ui'
# The existing scheduled task can run even when the guest rejects re-registering it.
& C:\weft\interactive.ps1 -Command "cd $checkout; pnpm --filter showcase shot:windows" -Timeout 900
if ((Get-Content C:\weft\interactive.log -Tail 1) -ne 'exit 0') { exit 1 }
Copy-Item "$checkout\showcase\screenshots\actual\windows\*.png" "$hostCheckout\showcase\screenshots\actual\windows" -ErrorAction Stop
