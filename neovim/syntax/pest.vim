" Vim syntax file
" Language: Pest grammar specification
" Maintainer: Samyak Sarnayak <samyak201@gmail.com>
" Last Change: 2023-05-31

if exists("b:current_syntax")
  finish
endif

let b:current_syntax = "pest"

setlocal foldmethod=syntax

" Keywords
syn keyword pestKeywords ANY SOI EOI PUSH PEEK POP DROP

" regions
syn region pestProductionRHS matchgroup=pestBraces start="{" end="}" fold transparent
syn region pestString start=/"/ skip=/\\\\\|\\"/ end=/"/ oneline
syn region pestChar start=/'/ skip=/\\'/ end=/'/ oneline

" stuff
syn match pestIdent "[[:alnum:]_]\+" display

syn match pestPunctuation '\~\|='

syn match pestSpecial display "@\|^\|?\|*\|!\|&\|#"

syn match pestComment "//.*$" contains=pestTodo
syn keyword pestTodo contained TODO FIXME XXX NB NOTE

highlight default link pestKeywords Keyword
highlight default link pestIdent Identifier
highlight default link pestPunctuation Delimiter
highlight default link pestBraces Delimiter
highlight default link pestSpecial Special
highlight default link pestComment Comment
highlight default link pestTodo Todo
highlight default link pestString String
highlight default link pestChar Character
