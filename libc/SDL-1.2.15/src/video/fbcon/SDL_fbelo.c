/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Lesser General Public
    License as published by the Free Software Foundation; either
    version 2.1 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public
    License along with this library; if not, write to the Free Software
    Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

#include <unistd.h>
#include <sys/time.h>
#include <ctype.h>

#include "SDL_stdinc.h"
#include "SDL_fbvideo.h"
#include "SDL_fbelo.h"

/*
	calibration default values
	values are read from the following environment variables:

	SDL_ELO_MIN_X
	SDL_ELO_MAX_X
	SDL_ELO_MIN_Y
	SDL_ELO_MAX_Y
*/

static int ELO_MIN_X = 400;
static int ELO_MAX_X = 3670;
static int ELO_MIN_Y = 500;
static int ELO_MAX_Y = 3540;

#define ELO_SNAP_SIZE 6
#define ELO_TOUCH_BYTE		'T'	
#define ELO_ID			'I'
#define ELO_MODE		'M'
#define ELO_PARAMETER		'P'
#define ELO_REPORT		'B'
#define ELO_ACK			'A'	

#define ELO_INIT_CHECKSUM	0xAA

#define ELO_BTN_PRESS		0x01	
#define ELO_STREAM		0x02
#define ELO_BTN_RELEASE		0x04

#define ELO_TOUCH_MODE		0x01
#define ELO_STREAM_MODE		0x02
#define ELO_UNTOUCH_MODE	0x04
#define ELO_RANGE_CHECK_MODE	0x40
#define ELO_TRIM_MODE		0x02
#define ELO_CALIB_MODE		0x04
#define ELO_SCALING_MODE	0x08
#define ELO_TRACKING_MODE	0x40

#define ELO_SERIAL_MASK		0xF8

#define ELO_SERIAL_IO		'0'

#define ELO_MAX_TRIALS	3
#define ELO_MAX_WAIT		100000
#define ELO_UNTOUCH_DELAY	5
#define ELO_REPORT_DELAY	1

/*	eloParsePacket
*/
int eloParsePacket(unsigned char* mousebuf, int* dx, int* dy, int* button_state) {
	static int elo_button = 0;
	static int last_x = 0;
	static int last_y = 0;
	int x,y;

	/* Check if we have a touch packet */
	if (mousebuf[1] != ELO_TOUCH_BYTE) {
		return 0;
	}

	x = ((mousebuf[4] << 8) | mousebuf[3]);
	y = ((mousebuf[6] << 8) | mousebuf[5]);

	if((SDL_abs(x - last_x) > ELO_SNAP_SIZE) || (SDL_abs(y - last_y) > ELO_SNAP_SIZE)) {
		*dx = ((mousebuf[4] << 8) | mousebuf[3]);
		*dy = ((mousebuf[6] << 8) | mousebuf[5]);
	}
	else {
		*dx = last_x;
		*dy = last_y;
	}

	last_x = *dx;
	last_y = *dy;

	if ( (mousebuf[2] & 0x07) == ELO_BTN_PRESS ) {
		elo_button = 1;
	}
	if ( (mousebuf[2] & 0x07) == ELO_BTN_RELEASE ) {
		elo_button = 0;
	}

	*button_state = elo_button;
	return 1;
}

/*	Convert the raw coordinates from the ELO controller
	to a screen position.
*/
void eloConvertXY(_THIS, int *dx,  int *dy) {
	int input_x = *dx;
	int input_y = *dy;
	int width = ELO_MAX_X - ELO_MIN_X;
	int height = ELO_MAX_Y - ELO_MIN_Y;

	*dx = ((int)cache_vinfo.xres - ((int)cache_vinfo.xres * (input_x - ELO_MIN_X)) / width);
	*dy = (cache_vinfo.yres * (input_y - ELO_MIN_Y)) / height;
}


/*	eloGetPacket
*/
int eloGetPacket(unsigned char* buffer, int* buffer_p, int* checksum, int fd) {
	int num_bytes;
	int ok;

	if(fd == 0) {
		num_bytes = ELO_PACKET_SIZE;
	}
	else {
		num_bytes = read(fd,
			(char *) (buffer + *buffer_p),
			ELO_PACKET_SIZE - *buffer_p);
	}

	if (num_bytes < 0) {
#ifdef DEBUG_MOUSE
		fprintf(stderr, "System error while reading from Elographics touchscreen.\n");
#endif
		return 0;
	}

	while (num_bytes) {
		if ((*buffer_p == 0) && (buffer[0] != ELO_START_BYTE)) {
			SDL_memcpy(&buffer[0], &buffer[1], num_bytes-1);
		}
		else {
			if (*buffer_p < ELO_PACKET_SIZE-1) {
				*checksum = *checksum + buffer[*buffer_p];
				*checksum = *checksum % 256;
			}
			(*buffer_p)++;
		}
		num_bytes--;
	}

	if (*buffer_p == ELO_PACKET_SIZE) {
		ok = (*checksum == buffer[ELO_PACKET_SIZE-1]);
		*checksum = ELO_INIT_CHECKSUM;
		*buffer_p = 0;

		if (!ok) {
			return 0;
		}

		return 1;
	}
	else {
		return 0;
	}
}

/* eloSendPacket
*/

int eloSendPacket(unsigned char* packet, int fd)
{
	int i, result;
	int sum = ELO_INIT_CHECKSUM;

	packet[0] = ELO_START_BYTE;
	for (i = 0; i < ELO_PACKET_SIZE-1; i++) {
		sum += packet[i];
		sum &= 0xFF;
	}
	packet[ELO_PACKET_SIZE-1] = sum;

	result = write(fd, packet, ELO_PACKET_SIZE);

	if (result != ELO_PACKET_SIZE) {
#ifdef DEBUG_MOUSE
		printf("System error while sending to Elographics touchscreen.\n");
#endif
		return 0;
	}
	else {
		return 1;
	}
}


/*	eloWaitForInput
 */
int eloWaitForInput(int fd, int timeout)
{
	fd_set readfds;
	struct timeval to;
	int r;

	FD_ZERO(&readfds);
	FD_SET(fd, &readfds);
	to.tv_sec = 0;
	to.tv_usec = timeout;

	r = select(FD_SETSIZE, &readfds, NULL, NULL, &to);
	return r;
}

/*	eloWaitReply
 */
int eloWaitReply(unsigned char type, unsigned char *reply, int fd) {
	int ok;
	int i, result;
	int reply_p = 0;
	int sum = ELO_INIT_CHECKSUM;

	i = ELO_MAX_TRIALS;
	do {
		ok = 0;

		result = eloWaitForInput(fd, ELO_MAX_WAIT);

		if (result > 0) {
			ok = eloGetPacket(reply, &reply_p, &sum, fd);

			if (ok && reply[1] != type && type != ELO_PARAMETER) {
#ifdef DEBUG_MOUSE
				fprintf(stderr, "Wrong reply received\n");
#endif
				ok = 0;
			}
		}
		else {
#ifdef DEBUG_MOUSE
			fprintf(stderr, "No input!\n");
#endif
		}

		if (result == 0) {
			i--;
		}
	} while(!ok && (i>0));

	return ok;
}


/*	eloWaitAck
 */

int eloWaitAck(int fd) {
	unsigned char packet[ELO_PACKET_SIZE];
	int i, nb_errors;

	if (eloWaitReply(ELO_ACK, packet, fd)) {
		for (i = 0, nb_errors = 0; i < 4; i++) {
			if (packet[2 + i] != '0') {
				nb_errors++;
			}
		}

		if (nb_errors != 0) {
#ifdef DEBUG_MOUSE
			fprintf(stderr, "Elographics acknowledge packet reports %d errors\n", nb_errors);
#endif
		}
		return 1;
	}
	else {
		return 0;
	}
}


/*	eloSendQuery --
*/
int eloSendQuery(unsigned char *request, unsigned char* reply, int fd) {
	int ok;

	if (eloSendPacket(request, fd)) {
		ok = eloWaitReply(toupper(request[1]), reply, fd);
		if (ok) {
			ok = eloWaitAck(fd);
		}
		return ok;
	}
	else {
		return 0;
	}
}


/*	eloSendControl
*/
int eloSendControl(unsigned char* control, int fd) {
	if (eloSendPacket(control, fd)) {
		return eloWaitAck(fd);
	}
	else {
		return 0;
	}
}

/*	eloInitController
*/
int eloInitController(int fd) {
	unsigned char req[ELO_PACKET_SIZE];
	unsigned char reply[ELO_PACKET_SIZE];
	const char *buffer = NULL;
	int result = 0;

	struct termios mouse_termios;

	/* try to read the calibration values */
	buffer = SDL_getenv("SDL_ELO_MIN_X");
	if(buffer) {
		ELO_MIN_X = SDL_atoi(buffer);
	}
	buffer = SDL_getenv("SDL_ELO_MAX_X");
	if(buffer) {
		ELO_MAX_X = SDL_atoi(buffer);
	}
	buffer = SDL_getenv("SDL_ELO_MIN_Y");
	if(buffer) {
		ELO_MIN_Y = SDL_atoi(buffer);
	}
	buffer = SDL_getenv("SDL_ELO_MAX_Y");
	if(buffer) {
		ELO_MAX_Y = SDL_atoi(buffer);
	}

#ifdef DEBUG_MOUSE
	fprintf( stderr, "ELO calibration values:\nmin_x: %i\nmax_x: %i\nmin_y: %i\nmax_y: %i\n",
		ELO_MIN_X,
		ELO_MAX_X,
		ELO_MIN_Y,
		ELO_MAX_Y);
#endif

	/* set comm params */
	SDL_memset(&mouse_termios, 0, sizeof(mouse_termios));
	mouse_termios.c_cflag = B9600 | CS8 | CREAD | CLOCAL;
	mouse_termios.c_cc[VMIN] = 1;
	result = tcsetattr(fd, TCSANOW, &mouse_termios);

	if (result < 0) {
#ifdef DEBUG_MOUSE
		fprintf( stderr, "Unable to configure Elographics touchscreen port\n");
#endif
		return 0;
	}

	SDL_memset(req, 0, ELO_PACKET_SIZE);
	req[1] = tolower(ELO_PARAMETER);
	if (!eloSendQuery(req, reply, fd)) {
#ifdef DEBUG_MOUSE
		fprintf( stderr, "Not at the specified rate or model 2310, will continue\n");
#endif
	}

	SDL_memset(req, 0, ELO_PACKET_SIZE);
	req[1] = tolower(ELO_ID);
	if (eloSendQuery(req, reply, fd)) {
#ifdef DEBUG_MOUSE
		fprintf(stderr, "Ok, controller configured!\n");
#endif
	}
	else {
#ifdef DEBUG_MOUSE
		fprintf( stderr, "Unable to ask Elographics touchscreen identification\n");
#endif
		return 0;
	}

	SDL_memset(req, 0, ELO_PACKET_SIZE);
	req[1] = ELO_MODE;
	req[3] = ELO_TOUCH_MODE | ELO_STREAM_MODE | ELO_UNTOUCH_MODE;
	req[4] = ELO_TRACKING_MODE;
	if (!eloSendControl(req, fd)) {
#ifdef DEBUG_MOUSE
		fprintf( stderr, "Unable to change Elographics touchscreen operating mode\n");
#endif
		return 0;
	}

	SDL_memset(req, 0, ELO_PACKET_SIZE);
	req[1] = ELO_REPORT;
	req[2] = ELO_UNTOUCH_DELAY;
	req[3] = ELO_REPORT_DELAY;
	if (!eloSendControl(req, fd)) {
#ifdef DEBUG_MOUSE
		fprintf( stderr, "Unable to change Elographics touchscreen reports timings\n");
#endif
		return 0;
	}

	return 1;
}

int eloReadPosition(_THIS, int fd, int* x, int* y, int* button_state, int* realx, int* realy) {
        unsigned char buffer[ELO_PACKET_SIZE];
        int pointer = 0;
        int checksum = ELO_INIT_CHECKSUM;

        while(pointer < ELO_PACKET_SIZE) {
                if(eloGetPacket(buffer, &pointer, &checksum, fd)) {
                        break;
                }
        }

        if(!eloParsePacket(buffer, realx, realy, button_state)) {
                return 0;
        }

        *x = *realx;
        *y = *realy;

        eloConvertXY(this, x, y);
	
	return 1;
}
