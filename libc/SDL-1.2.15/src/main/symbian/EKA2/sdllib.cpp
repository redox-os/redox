#include<eikstart.h>
#include<sdlmain.h>
#include<sdlepocapi.h>


GLREF_C TInt E32Main()
    {
    return SDLEnv::SetMain(SDL_main, CSDL::EEnableFocusStop | CSDL::EAllowImageResize,
     NULL, SDLEnv::EParamQuery | SDLEnv::EVirtualMouse);
    }
    
    