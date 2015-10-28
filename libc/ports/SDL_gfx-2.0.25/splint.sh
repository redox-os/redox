#
# Splint based static code analysis
# http://www.splint.org/
#

splint -I/usr/local/include/SDL -exportlocal SDL_framerate.c

# splint -I/usr/local/include/SDL -weak +matchanyintegral SDL_gfxBlitFunc.c
splint -I/usr/local/include/SDL -exportlocal +matchanyintegral -shiftimplementation -boolops +boolint -predboolint -shadow -casebreak -dependenttrans -nullstate -compdestroy -compmempass SDL_gfxBlitFunc.c

# splint -I/usr/local/include/SDL -weak +matchanyintegral SDL_rotozoom.c
splint -I/usr/local/include/SDL -exportlocal +matchanyintegral -shiftimplementation -boolops +boolint -retvalint -predboolint -branchstate -nullret -paramuse -compdef -nullpass -usedef -mustfreefresh -incondefs SDL_rotozoom.c

# splint -I/usr/local/include/SDL -weak +matchanyintegral SDL_gfxPrimitives.c
splint -I/usr/local/include/SDL -exportlocal +matchanyintegral -shiftimplementation -boolops -type -retvalint -shiftnegative -predboolint -nullassign -branchstate -globstate -usereleased -nullstate -nullpass -compdef -usedef -evalorder -compdestroy SDL_gfxPrimitives.c

# splint -I/usr/local/include/SDL -weak +matchanyintegral SDL_imageFilter.c
splint -I/usr/local/include/SDL -exportlocal +matchanyintegral -shiftimplementation -boolops +boolint -paramuse -retvalint +charint -mayaliasunique SDL_imageFilter.c
