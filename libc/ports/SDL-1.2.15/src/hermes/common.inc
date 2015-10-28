; Some common macros for hermes nasm code

%macro SDL_FUNC 1
%ifdef HIDDEN_VISIBILITY
GLOBAL %1:function hidden
%else
GLOBAL %1
%endif
%endmacro
