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

#include "SDL_QWin.h"
#include <qapplication.h>
#include <qdirectpainter_qws.h>

screenRotationT screenRotation = SDL_QT_NO_ROTATION;

SDL_QWin::SDL_QWin(const QSize& size)
  : QWidget(0, "SDL_main"), my_painter(0), my_image(0),
    my_inhibit_resize(false), my_mouse_pos(-1,-1), my_flags(0),
    my_has_fullscreen(false), my_locked(0)
{
  setBackgroundMode(NoBackground);
}

SDL_QWin::~SDL_QWin() {
  // Nothing to do yet.
  if(my_image) {
    delete my_image;
  }
}

void SDL_QWin::setImage(QImage *image) {
  if ( my_image ) {
    delete my_image;
  }
  my_image = image;
  //  setFixedSize(image->size());
}

void SDL_QWin::resizeEvent(QResizeEvent *e) {
  if(size() != qApp->desktop()->size()) {
    // Widget is not the correct size, so do the fullscreen magic
    my_has_fullscreen = false;
    enableFullscreen();
  }
  if(my_inhibit_resize) {
    my_inhibit_resize = false;
  } else {
    SDL_PrivateResize(e->size().width(), e->size().height());
  }
}

void SDL_QWin::focusInEvent(QFocusEvent *) {
  // Always do it here, no matter the size.
  enableFullscreen();
  SDL_PrivateAppActive(true, SDL_APPINPUTFOCUS);
}

void SDL_QWin::focusOutEvent(QFocusEvent *) {
  my_has_fullscreen = false;
  SDL_PrivateAppActive(false, SDL_APPINPUTFOCUS);
}

void SDL_QWin::closeEvent(QCloseEvent *e) {
  SDL_PrivateQuit();
  e->ignore();
}

void SDL_QWin::setMousePos(const QPoint &pos) {
  if(my_image->width() == height()) {
    if (screenRotation == SDL_QT_ROTATION_90)
      my_mouse_pos = QPoint(height()-pos.y(), pos.x());
    else if (screenRotation == SDL_QT_ROTATION_270)
      my_mouse_pos = QPoint(pos.y(), width()-pos.x());
  } else {
    my_mouse_pos = pos;
  }
}

void SDL_QWin::mouseMoveEvent(QMouseEvent *e) {
  Qt::ButtonState button = e->button();
  int sdlstate = 0;
  if( (button & Qt::LeftButton)) {
    sdlstate |= SDL_BUTTON_LMASK;
  }
  if( (button & Qt::RightButton)) {
    sdlstate |= SDL_BUTTON_RMASK;
  }
  if( (button & Qt::MidButton)) {
    sdlstate |= SDL_BUTTON_MMASK;
  }
  setMousePos(e->pos());
  SDL_PrivateMouseMotion(sdlstate, 0, my_mouse_pos.x(), my_mouse_pos.y());
}

void SDL_QWin::mousePressEvent(QMouseEvent *e) {
  mouseMoveEvent(e);
  Qt::ButtonState button = e->button();
  SDL_PrivateMouseButton(SDL_PRESSED,
			 (button & Qt::LeftButton) ? 1 :
			 ((button & Qt::RightButton) ? 2 : 3),
			 my_mouse_pos.x(), my_mouse_pos.y());
}

void SDL_QWin::mouseReleaseEvent(QMouseEvent *e) {
  setMousePos(e->pos());
  Qt::ButtonState button = e->button();
  SDL_PrivateMouseButton(SDL_RELEASED,
			 (button & Qt::LeftButton) ? 1 :
			 ((button & Qt::RightButton) ? 2 : 3),
			 my_mouse_pos.x(), my_mouse_pos.y());
  my_mouse_pos = QPoint(-1, -1);
}

static inline void
gs_fastRotateBlit_3 ( unsigned short *fb,
		      unsigned short *bits,
		      const QRect& rect )
{
  // FIXME: this only works correctly for 240x320 displays
  int startx, starty;
  int width, height;
  
  startx = rect.left() >> 1;
  starty = rect.top() >> 1;
  width  = ((rect.right() - rect.left()) >> 1) + 2;
  height = ((rect.bottom() - rect.top()) >> 1) + 2;

  if((startx+width) > 120) {
    width = 120 - startx; // avoid horizontal overflow
  }
  if((starty+height) > 160) { 
    height = 160 - starty; // avoid vertical overflow
  }

  ulong *sp1, *sp2, *dp1, *dp2;
  ulong stop, sbot, dtop, dbot;    
  
  sp1 = (ulong*)bits + startx + starty*240;
  sp2 = sp1 + 120;
  dp1 = (ulong *)fb + (159 - starty) + startx*320;
  dp2 = dp1 + 160;
  int rowadd = (-320*width) - 1;
  int rowadd2 = 240 - width;
  // transfer in cells of 2x2 pixels in words
  for (int y=0; y<height; y++) {
    for (int x=0; x<width; x++) {
      // read source pixels
      stop = *sp1;
      sbot = *sp2;
      // rotate pixels
      dtop = (sbot & 0xffff) + ((stop & 0xffff)<<16);
      dbot = ((sbot & 0xffff0000)>>16) + (stop & 0xffff0000);
      // write to framebuffer
      *dp1 = dtop;
      *dp2 = dbot;
      // update source ptrs
      sp1++; sp2++;
      // update dest ptrs - 2 pix at a time
      dp1 += 320;
      dp2 += 320;
    }
    // adjust src ptrs - skip a row as we work in pairs
    sp1 += rowadd2;
    sp2 += rowadd2;
    // adjust dest ptrs for rotation
    dp1 += rowadd;
    dp2 += rowadd;
  }
}

static inline void
gs_fastRotateBlit_1 ( unsigned short *fb,
		      unsigned short *bits,
		      const QRect& rect ) {
  // FIXME: this only works correctly for 240x320 displays
  int startx, starty;
  int width, height;

  startx = rect.left() >> 1;
  starty = rect.top() >> 1;
  width  = ((rect.right() - rect.left()) >> 1) + 2;
  height = ((rect.bottom() - rect.top()) >> 1) + 2;

  if((startx+width) > 120) {
    width = 120 - startx; // avoid horizontal overflow
  }
  if((starty+height) > 160) { 
    height = 160 - starty; // avoid vertical overflow
  }

  ulong *sp1, *sp2, *dp1, *dp2;
  ulong stop, sbot, dtop, dbot;    
  fb += 320*239; // Move "fb" to top left corner
  sp1 = (ulong*)bits + startx + starty*240;
  sp2 = sp1 + 120;
  dp1 = (ulong*)fb - startx * 320 - starty;
  dp2 = dp1 - 160;
  int rowadd = (320*width) + 1;
  int rowadd2 = 240 - width;
  // transfer in cells of 2x2 pixels in words
  for (int y=0; y<height; y++) {
    for (int x=0; x<width; x++) {
      // read
      stop = *sp1;
      sbot = *sp2;
      // rotate
      dtop = (stop & 0xffff) + ((sbot & 0xffff)<<16);
      dbot = ((stop & 0xffff0000)>>16) + (sbot & 0xffff0000);
      // write
      *dp1 = dtop;
      *dp2 = dbot;
      // update source ptrs
      sp1++; sp2++;
      // update dest ptrs - 2 pix at a time
      dp1 -= 320;
      dp2 -= 320;
    }
    // adjust src ptrs - skip a row as we work in pairs
    sp1 += rowadd2;
    sp2 += rowadd2;
    // adjust dest ptrs for rotation
    dp1 += rowadd;
    dp2 += rowadd;
  }
}

// desktop, SL-A300 etc
bool SDL_QWin::repaintRotation0(const QRect& rect) {
  if(my_image->width() == width()) {
    uchar *fb = (uchar*)my_painter->frameBuffer();
    uchar *buf = (uchar*)my_image->bits();
    if(rect == my_image->rect()) {
      SDL_memcpy(fb, buf, width()*height()*2);
    } else {
      int h = rect.height();
      int wd = rect.width()<<1;
      int fblineadd = my_painter->lineStep();
      int buflineadd = my_image->bytesPerLine();
      fb  += (rect.left()<<1) + rect.top() * my_painter->lineStep();
      buf += (rect.left()<<1) + rect.top() * my_image->bytesPerLine();
      while(h--) {
	SDL_memcpy(fb, buf, wd);
	fb += fblineadd;
	buf += buflineadd;
      }
    }
  } else {
    return false; // FIXME: Landscape
  }
#ifdef __i386__
  my_painter->fillRect( rect, QBrush( Qt::NoBrush ) );
#endif
  return true;
}

  
// Sharp Zaurus SL-5500 etc 
bool SDL_QWin::repaintRotation3(const QRect& rect) {
  if(my_image->width() == width()) {
    ushort *fb = (ushort*)my_painter->frameBuffer();
    ushort *buf = (ushort*)my_image->bits();
    gs_fastRotateBlit_3(fb, buf, rect);
  } else {
    // landscape mode
    if (screenRotation == SDL_QT_ROTATION_90) {
      uchar *fb = (uchar*)my_painter->frameBuffer();
      uchar *buf = (uchar*)my_image->bits();
      if(rect == my_image->rect()) {
	SDL_memcpy(fb, buf, width()*height()*2);
      } else {
	int h = rect.height();
	int wd = rect.width()<<1;
	int fblineadd = my_painter->lineStep();
	int buflineadd = my_image->bytesPerLine();
	fb  += (rect.left()<<1) + rect.top() * my_painter->lineStep();
	buf += (rect.left()<<1) + rect.top() * my_image->bytesPerLine();
	while(h--) {
	  SDL_memcpy(fb, buf, wd);
	  fb += fblineadd;
	  buf += buflineadd;
	}
      }
    } else if (screenRotation == SDL_QT_ROTATION_270) {
      int h = rect.height();
      int wd = rect.width();
      int fblineadd = my_painter->lineStep() - (rect.width() << 1);
      int buflineadd = my_image->bytesPerLine() - (rect.width() << 1);
      int w;

      uchar *fb = (uchar*)my_painter->frameBuffer();
      uchar *buf = (uchar*)my_image->bits();
        
      fb += ((my_painter->width() - (rect.top() + rect.height())) * 
	     my_painter->lineStep()) + ((my_painter->height() - ((rect.left() + 
								  rect.width()))) << 1);

      buf += my_image->bytesPerLine() * (rect.top() + rect.height()) -
	(((my_image->width() - (rect.left() + rect.width())) << 1) + 2);

      while(h--) {
	w = wd;
	while(w--) *((unsigned short*)fb)++ = *((unsigned short*)buf)--;
	fb += fblineadd;
	buf -= buflineadd;
      }
    }
  }
  return true;
}

// ipaq 3800...
bool SDL_QWin::repaintRotation1(const QRect& rect) {
  if(my_image->width() == width()) {
    ushort *fb = (ushort*)my_painter->frameBuffer();
    ushort *buf = (ushort*)my_image->bits();
    gs_fastRotateBlit_1(fb, buf, rect);
  } else {
    return false; // FIXME: landscape mode
  }
  return true;
}

void SDL_QWin::repaintRect(const QRect& rect) {
  if(!my_painter || !rect.width() || !rect.height()) {
    return;
  }
  
  if(QPixmap::defaultDepth() == 16) {
    switch(my_painter->transformOrientation()) {
    case 3:
      if(repaintRotation3(rect)) { return;  }
      break;
    case 1:
      if(repaintRotation1(rect)) { return;  }
      break;
    case 0:
      if(repaintRotation0(rect)) { return;  }
      break;
    }
  } 
  my_painter->drawImage(rect.topLeft(), *my_image, rect);
}

// This paints the current buffer to the screen, when desired. 
void SDL_QWin::paintEvent(QPaintEvent *ev) {  
  if(my_image) {
    lockScreen(true);
    repaintRect(ev->rect());
    unlockScreen();
  }
}  

/* Function to translate a keyboard transition and queue the key event
 * This should probably be a table although this method isn't exactly
 * slow.
 */
void SDL_QWin::QueueKey(QKeyEvent *e, int pressed)
{  
  SDL_keysym keysym;
  int scancode = e->key();
  /* Set the keysym information */
  if(scancode >= 'A' && scancode <= 'Z') {
    // Qt sends uppercase, SDL wants lowercase
    keysym.sym = static_cast<SDLKey>(scancode + 32);
  } else if(scancode  >= 0x1000) {
    // Special keys
    switch(scancode) {
    case Qt::Key_Escape: scancode = SDLK_ESCAPE; break;
    case Qt::Key_Tab: scancode = SDLK_TAB; break;
    case Qt::Key_Backspace: scancode = SDLK_BACKSPACE; break;
    case Qt::Key_Return: scancode = SDLK_RETURN; break;
    case Qt::Key_Enter: scancode = SDLK_KP_ENTER; break;
    case Qt::Key_Insert: scancode = SDLK_INSERT; break;
    case Qt::Key_Delete: scancode = SDLK_DELETE; break;
    case Qt::Key_Pause: scancode = SDLK_PAUSE; break;
    case Qt::Key_Print: scancode = SDLK_PRINT; break;
    case Qt::Key_SysReq: scancode = SDLK_SYSREQ; break;
    case Qt::Key_Home: scancode = SDLK_HOME; break;
    case Qt::Key_End: scancode = SDLK_END; break;
    // We want the control keys to rotate with the screen
    case Qt::Key_Left: 
        if (screenRotation == SDL_QT_ROTATION_90) scancode = SDLK_UP;
        else if (screenRotation == SDL_QT_ROTATION_270) scancode = SDLK_DOWN;
        else scancode = SDLK_LEFT;
        break;
    case Qt::Key_Up: 
        if (screenRotation == SDL_QT_ROTATION_90) scancode = SDLK_RIGHT;
        else if (screenRotation == SDL_QT_ROTATION_270) scancode = SDLK_LEFT;
        else scancode = SDLK_UP;
        break;
    case Qt::Key_Right: 
        if (screenRotation == SDL_QT_ROTATION_90) scancode = SDLK_DOWN;
        else if (screenRotation == SDL_QT_ROTATION_270) scancode = SDLK_UP;
        else scancode = SDLK_RIGHT;
        break;
    case Qt::Key_Down:
        if (screenRotation == SDL_QT_ROTATION_90) scancode = SDLK_LEFT;
        else if (screenRotation == SDL_QT_ROTATION_270) scancode = SDLK_RIGHT;
        else scancode = SDLK_DOWN;
        break;
    case Qt::Key_Prior: scancode = SDLK_PAGEUP; break;
    case Qt::Key_Next: scancode = SDLK_PAGEDOWN; break;
    case Qt::Key_Shift: scancode = SDLK_LSHIFT; break;
    case Qt::Key_Control: scancode = SDLK_LCTRL; break;
    case Qt::Key_Meta: scancode = SDLK_LMETA; break;
    case Qt::Key_Alt: scancode = SDLK_LALT; break;
    case Qt::Key_CapsLock: scancode = SDLK_CAPSLOCK; break;
    case Qt::Key_NumLock: scancode = SDLK_NUMLOCK; break;
    case Qt::Key_ScrollLock: scancode = SDLK_SCROLLOCK; break;
    case Qt::Key_F1: scancode = SDLK_F1; break;
    case Qt::Key_F2: scancode = SDLK_F2; break;
    case Qt::Key_F3: scancode = SDLK_F3; break;
    case Qt::Key_F4: scancode = SDLK_F4; break;
    case Qt::Key_F5: scancode = SDLK_F5; break;
    case Qt::Key_F6: scancode = SDLK_F6; break;
    case Qt::Key_F7: scancode = SDLK_F7; break;
    case Qt::Key_F8: scancode = SDLK_F8; break;
    case Qt::Key_F9: scancode = SDLK_F9; break;
    case Qt::Key_F10: scancode = SDLK_F10; break;
    case Qt::Key_F11: scancode = SDLK_F11; break;
    case Qt::Key_F12: scancode = SDLK_F12; break;
    case Qt::Key_F13: scancode = SDLK_F13; break;
    case Qt::Key_F14: scancode = SDLK_F14; break;
    case Qt::Key_F15: scancode = SDLK_F15; break;
    case Qt::Key_Super_L: scancode = SDLK_LSUPER; break;
    case Qt::Key_Super_R: scancode = SDLK_RSUPER; break;
    case Qt::Key_Menu: scancode = SDLK_MENU; break;
    case Qt::Key_Help: scancode = SDLK_HELP; break;

    case Qt::Key_F33:
      // FIXME: This is a hack to enable the OK key on
      // Zaurii devices. SDLK_RETURN is a suitable key to use
      // since it often is used as such.
      //     david@hedbor.org
      scancode = SDLK_RETURN;
      break;
    default:
      scancode = SDLK_UNKNOWN;
      break;
    }
    keysym.sym = static_cast<SDLKey>(scancode);    
  } else {
    keysym.sym = static_cast<SDLKey>(scancode);    
  }
  keysym.scancode = scancode;
  keysym.mod = KMOD_NONE;
  ButtonState st = e->state();
  if( (st & ShiftButton) )   { keysym.mod = static_cast<SDLMod>(keysym.mod | KMOD_LSHIFT);  }
  if( (st & ControlButton) ) { keysym.mod = static_cast<SDLMod>(keysym.mod | KMOD_LCTRL);  }
  if( (st & AltButton) )     { keysym.mod = static_cast<SDLMod>(keysym.mod | KMOD_LALT);  }
  if ( SDL_TranslateUNICODE ) {
    QChar qchar = e->text()[0];
    keysym.unicode = qchar.unicode();
  } else {
    keysym.unicode = 0;
  }

  /* NUMLOCK and CAPSLOCK are implemented as double-presses in reality */
  //	if ( (keysym.sym == SDLK_NUMLOCK) || (keysym.sym == SDLK_CAPSLOCK) ) {
  //		pressed = 1;
  //	}

  /* Queue the key event */
  if ( pressed ) {
    SDL_PrivateKeyboard(SDL_PRESSED, &keysym);
  } else {
    SDL_PrivateKeyboard(SDL_RELEASED, &keysym);
  }
}

void SDL_QWin::setFullscreen(bool fs_on) {
  my_has_fullscreen = false;
  enableFullscreen();
}

void SDL_QWin::enableFullscreen() {
  // Make sure size is correct
  if(!my_has_fullscreen) {
    setFixedSize(qApp->desktop()->size());
    // This call is needed because showFullScreen won't work
    // correctly if the widget already considers itself to be fullscreen.
    showNormal();
    // This is needed because showNormal() forcefully changes the window
    // style to WSTyle_TopLevel.
    setWFlags(WStyle_Customize | WStyle_NoBorder);
    // Enable fullscreen.
    showFullScreen();
    my_has_fullscreen = true;
  }
}

bool SDL_QWin::lockScreen(bool force) {
  if(!my_painter) {
    if(force || (isVisible() && isActiveWindow())) {
      my_painter = new QDirectPainter(this);
    } else {
      return false;
    }
  }
  my_locked++; // Increate lock refcount
  return true;
}

void SDL_QWin::unlockScreen() {
  if(my_locked > 0) {
    my_locked--; // decrease lock refcount;
  }
  if(!my_locked && my_painter) {
    my_painter->end();
    delete my_painter;
    my_painter = 0;
  }
}
