kilobytes() {
    [ -d "$1" ] || { echo 0; return; }
    du -sk "$1" 2>/dev/null | awk '{ print $1 } END { if (NR == 0) print 0 }'
}

human() {
    awk -v kb="$1" 'BEGIN {
        if (kb < 0) kb = 0
        if (kb >= 1048576) printf "%.1f ГБ", kb / 1048576
        else printf "%.0f МБ", kb / 1024
    }'
}
