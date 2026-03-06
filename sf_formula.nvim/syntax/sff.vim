" Quit if syntax already loaded for this buffer
if exists("b:current_syntax")
  finish
endif

" --- Boolean literals ---
syn keyword sffBoolean True False
hi def link sffBoolean Boolean

" --- Numbers ---
syn match sffNumber '\v\d+(\.\d+)?' contains=sffNumberDot
syn match sffNumberDot '\.' contained
hi def link sffNumber Number

" --- Operators ---
syn match sffOperator '\(==\|!=\|<>\|<=\|>=\|&&\||||\)'
syn match sffOperator "[-^*/+=<>&]"
hi def link sffOperator Operator

" --- Identifier ---
syn match sffIdent '\([A-Za-z][A-Za-z0-9_]*\)'
hi def link sffIdent Ident

" --- Strings ---
" Double-quoted strings with escapes
syn region sffString start=+"+ skip=+""+ end=+"+
hi def link sffString String

" --- Block comments ---
syn region sffComment start="/\*" end="\*/" " contains=@Spell
hi def link sffComment Comment

" --- Function names (simple heuristic) ---
" Highlights an identifier right after 'fn'
syn match sffFunction "\v<(fn)\s+[A-Za-z_]\w*"
hi def link sffFunction Function

" --- Delimeter ---
syn match sffDelimiter "[(),.]"
hi def link sffDelimiter Delimiter

" --- TODO/FIXME inside comments ---
syn keyword sffTodo TODO FIXME XXX contained
hi def link sffTodo Todo
" Include it inside comments:
syn region sffComment start="/\*" end="\*/" contains=mylangTodo

let b:current_syntax = "sff"
