" Don't re-run if already loaded
if exists("b:did_ftplugin")
  finish
endif
let b:did_ftplugin = 1

" Example: line comment style
setlocal commentstring=/*\ %s\ */

" Example: indentation behavior (adjust later)
setlocal shiftwidth=4 tabstop=4 expandtab
