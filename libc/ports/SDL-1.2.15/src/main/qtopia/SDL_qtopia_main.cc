
/* Include the SDL main definition header */
#include "SDL_main.h"
#include <stdlib.h>
#include <unistd.h>
#ifdef main
#undef main
#endif
#ifdef QWS
#include <qpe/qpeapplication.h>
#include <qapplication.h>
#include <qpe/qpeapplication.h>
#include <stdlib.h>

// Workaround for OPIE to remove taskbar icon. Also fixes
// some issues in Qtopia where there are left-over qcop files in /tmp/.
// I'm guessing this will also clean up the taskbar in the Sharp version
// of Qtopia.
static inline void cleanupQCop() {
  QString appname(qApp->argv()[0]);
  int slash = appname.findRev("/");
  if(slash != -1) {  appname = appname.mid(slash+1); }
  QString cmd = QPEApplication::qpeDir() + "bin/qcop QPE/System 'closing(QString)' '"+appname+"'";
  system(cmd.latin1());
  cmd = "/tmp/qcop-msg-"+appname;
  unlink(cmd.latin1());
}

static QPEApplication *app;
#endif

extern int SDL_main(int argc, char *argv[]);

int main(int argc, char *argv[])
{
#ifdef QWS
  // This initializes the Qtopia application. It needs to be done here
  // because it parses command line options.
  app = new QPEApplication(argc, argv);
  QWidget dummy;
  app->showMainWidget(&dummy);
  atexit(cleanupQCop);
#endif
  // Exit here because if return is used, the application
  // doesn't seem to quit correctly.
  exit(SDL_main(argc, argv));
}
