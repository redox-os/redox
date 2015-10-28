
EXPORTS = SDL.x
HEADERS = \
	../../../../include/SDL.h \
	../../../../include/SDL_active.h \
	../../../../include/SDL_audio.h \
	../../../../include/SDL_byteorder.h \
	../../../../include/SDL_cdrom.h \
	../../../../include/SDL_copying.h \
	../../../../include/SDL_cpuinfo.h \
	../../../../include/SDL_endian.h \
	../../../../include/SDL_error.h \
	../../../../include/SDL_events.h \
	../../../../include/SDL_getenv.h \
	../../../../include/SDL_joystick.h \
	../../../../include/SDL_keyboard.h \
	../../../../include/SDL_keysym.h \
	../../../../include/SDL_loadso.h \
	../../../../include/SDL_mouse.h \
	../../../../include/SDL_mutex.h \
	../../../../include/SDL_name.h \
	../../../../include/SDL_platform.h \
	../../../../include/SDL_quit.h \
	../../../../include/SDL_rwops.h \
	../../../../include/SDL_syswm.h \
	../../../../include/SDL_thread.h \
	../../../../include/SDL_timer.h \
	../../../../include/SDL_types.h \
	../../../../include/SDL_version.h \
	../../../../include/SDL_video.h


all: $(EXPORTS)

$(EXPORTS): Makefile gendef.pl $(HEADERS)
	perl gendef.pl $(HEADERS) >$@ || rm $@

clean:
	rm -f $(EXPORTS)
