let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
let NvimTreeSetup =  1 
let Tabline_session_data = "[{\"show_all_buffers\": true, \"name\": \"Compile\", \"allowed_buffers\": []}, {\"show_all_buffers\": true, \"name\": \"Parse\", \"allowed_buffers\": []}, {\"show_all_buffers\": true, \"name\": \"TOML\", \"allowed_buffers\": []}]"
let NvimTreeRequired =  1 
silent only
silent tabonly
cd ~/Desktop/Projekte/PATRISPREDICTUM/Projekte/MMGX
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
if &shortmess =~ 'A'
  set shortmess=aoOA
else
  set shortmess=aoO
endif
badd +1 src/main.rs
badd +10 Cargo.toml
badd +2 ~/AppData/Local/nvim/coc-settings.json
badd +11 test.x
badd +1261 term://~/Desktop/Projekte/PATRISPREDICTUM/Projekte/MMGX//9496:C:/Windows/system32/cmd.exe
badd +2 src/compile.rs
badd +676 src/compile/parse.rs
badd +1 target/debug/deps/libMMGX-badc1557b6dc043a.rmeta
badd +1 test.c
badd +1934 ~/.rustup/toolchains/nightly-x86_64-pc-windows-msvc/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs
argglobal
%argdel
tabnew +setlocal\ bufhidden=wipe
tabnew +setlocal\ bufhidden=wipe
tabrewind
edit src/compile.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 30 + 146) / 293)
exe 'vert 2resize ' . ((&columns * 262 + 146) / 293)
argglobal
enew
file NvimTree_1
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
wincmd w
argglobal
balt src/compile/parse.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 2 - ((1 * winheight(0) + 40) / 80)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 2
normal! 093|
wincmd w
exe 'vert 1resize ' . ((&columns * 30 + 146) / 293)
exe 'vert 2resize ' . ((&columns * 262 + 146) / 293)
tabnext
edit src/compile/parse.rs
argglobal
balt ~/.rustup/toolchains/nightly-x86_64-pc-windows-msvc/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 676 - ((54 * winheight(0) + 40) / 80)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 676
normal! 032|
tabnext
edit src/main.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
split
1wincmd k
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe '1resize ' . ((&lines * 39 + 41) / 83)
exe 'vert 1resize ' . ((&columns * 146 + 146) / 293)
exe '2resize ' . ((&lines * 39 + 41) / 83)
exe 'vert 2resize ' . ((&columns * 146 + 146) / 293)
exe '3resize ' . ((&lines * 40 + 41) / 83)
argglobal
balt test.x
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 1 - ((0 * winheight(0) + 19) / 39)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 1
normal! 018|
wincmd w
argglobal
if bufexists(fnamemodify("Cargo.toml", ":p")) | buffer Cargo.toml | else | edit Cargo.toml | endif
if &buftype ==# 'terminal'
  silent file Cargo.toml
endif
balt test.c
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 9 - ((8 * winheight(0) + 19) / 39)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 9
normal! 07|
wincmd w
argglobal
if bufexists(fnamemodify("term://~/Desktop/Projekte/PATRISPREDICTUM/Projekte/MMGX//9496:C:/Windows/system32/cmd.exe", ":p")) | buffer term://~/Desktop/Projekte/PATRISPREDICTUM/Projekte/MMGX//9496:C:/Windows/system32/cmd.exe | else | edit term://~/Desktop/Projekte/PATRISPREDICTUM/Projekte/MMGX//9496:C:/Windows/system32/cmd.exe | endif
if &buftype ==# 'terminal'
  silent file term://~/Desktop/Projekte/PATRISPREDICTUM/Projekte/MMGX//9496:C:/Windows/system32/cmd.exe
endif
balt test.x
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
let s:l = 4096 - ((39 * winheight(0) + 20) / 40)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 4096
normal! 060|
wincmd w
3wincmd w
exe '1resize ' . ((&lines * 39 + 41) / 83)
exe 'vert 1resize ' . ((&columns * 146 + 146) / 293)
exe '2resize ' . ((&lines * 39 + 41) / 83)
exe 'vert 2resize ' . ((&columns * 146 + 146) / 293)
exe '3resize ' . ((&lines * 40 + 41) / 83)
tabnext 3
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let &winminheight = s:save_winminheight
let &winminwidth = s:save_winminwidth
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
set hlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
